use clap::Parser;
use serde::Deserialize;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread::sleep;
use std::time::{Duration, Instant};

#[derive(Parser)]
struct Args {
    /// Config file (TOML)
    #[arg(long)]
    config: Option<String>,

    /// DP832 IP address
    #[arg(long)]
    ip: Option<String>,

    /// SCPI port
    #[arg(long)]
    port: Option<u16>,

    /// Battery profile JSON
    #[arg(short, long)]
    profile: Option<String>,

    /// CSV log file
    #[arg(long)]
    log: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OcvPoint {
    soc: f64,
    voltage: f64,
}

#[derive(Debug, Deserialize)]
struct BatteryProfile {
    name: String,
    channel: u8,

    capacity_ah: f64,
    internal_resistance_ohm: f64,

    current_limit_discharge_a: f64,
    current_limit_charge_a: f64,

    cutoff_voltage: f64,
    max_voltage: f64,

    rc_time_constant_ms: u64,
    update_interval_ms: u64,

    ocv_curve: Vec<OcvPoint>,
}

#[derive(Debug, Deserialize, Default)]
struct Config {
    device: Option<DeviceConfig>,
    battery: Option<BatteryConfig>,
    logging: Option<LoggingConfig>,
}

#[derive(Debug, Deserialize)]
struct DeviceConfig {
    ip: String,
    port: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct BatteryConfig {
    profile: String,
}

#[derive(Debug, Deserialize)]
struct LoggingConfig {
    csv: Option<String>,
}

fn load_config(path: &str) -> Config {
    let mut s = String::new();
    std::fs::File::open(path)
        .unwrap()
        .read_to_string(&mut s)
        .unwrap();
    toml::from_str(&s).expect("Invalid config file")
}

fn default_config_path() -> Option<std::path::PathBuf> {
    let base = dirs_next::config_dir()?;
    Some(base.join("dp832-battery").join("config.toml"))
}

fn load_optional_config(path: Option<&str>) -> Config {
    let path = if let Some(p) = path {
        Some(std::path::PathBuf::from(p))
    } else {
        default_config_path()
    };

    if let Some(path) = path {
        if path.exists() {
            println!("Using config file: {}", path.display());
            let mut s = String::new();
            std::fs::File::open(path)
                .unwrap()
                .read_to_string(&mut s)
                .unwrap();
            toml::from_str(&s).expect("Invalid config file")
        } else {
            Config::default()
        }
    } else {
        Config::default()
    }
}

/* ---------------- SCPI helpers ---------------- */

fn send(stream: &mut TcpStream, cmd: &str) {
    let cmd = format!("{}\n", cmd);
    stream.write_all(cmd.as_bytes()).unwrap();
}

fn query(stream: &mut TcpStream, cmd: &str) -> String {
    send(stream, cmd);
    let mut resp = Vec::new();
    let mut buf = [0u8; 64];

    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                resp.extend_from_slice(&buf[..n]);
                if resp.ends_with(b"\n") {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(e) => panic!("{}", e),
        }
    }

    String::from_utf8_lossy(&resp).trim().to_string()
}

/* ---------------- Battery model ---------------- */

fn interpolate_ocv(curve: &[OcvPoint], soc: f64) -> f64 {
    let soc = soc.clamp(0.0, 1.0);

    for w in curve.windows(2) {
        if soc <= w[0].soc && soc >= w[1].soc {
            let t = (soc - w[1].soc) / (w[0].soc - w[1].soc);
            return w[1].voltage + t * (w[0].voltage - w[1].voltage);
        }
    }

    curve.last().unwrap().voltage
}

/* ---------------- Main ---------------- */

fn main() {
    let args = Args::parse();

    let cfg = load_optional_config(args.config.as_deref());
    // Resolve IP
    let ip = args
        .ip
        .or_else(|| cfg.device.as_ref().map(|d| d.ip.clone()))
        .unwrap_or_else(|| "192.168.1.100".to_string());

    // Resolve port
    let port = args
        .port
        .or_else(|| cfg.device.as_ref().and_then(|d| d.port))
        .unwrap_or(5555);

    // Resolve battery profile
    let profile_path = args
        .profile
        .or_else(|| cfg.battery.map(|b| b.profile))
        .expect("Battery profile not specified");

    // Resolve CSV log
    let csv_log = args
        .log
        .or_else(|| cfg.logging.and_then(|l| l.csv));

    println!("DP832: {}:{}", ip, port);
    println!("Profile: {}", profile_path);

    let mut json = String::new();
    File::open(&profile_path)
        .unwrap()
        .read_to_string(&mut json)
        .unwrap();

    let profile: BatteryProfile = serde_json::from_str(&json).unwrap();
    println!("Loaded profile: {}", profile.name);


    let addr = format!("{}:{}", ip, port);
    let mut stream = TcpStream::connect(&addr).unwrap();

    stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();

    send(&mut stream, "*CLS");
    println!("{}", query(&mut stream, "*IDN?"));

    send(&mut stream, &format!("INST:NSEL {}", profile.channel));
    send(&mut stream, "OUTP OFF");
    send(
        &mut stream,
        &format!("CURR {}", profile.current_limit_discharge_a),
    );

    send(&mut stream, "OUTP ON");

    let mut soc = 1.0;
    let mut last = Instant::now();
    let mut v_filt = interpolate_ocv(&profile.ocv_curve, soc);

    let mut csv = csv_log.map(|p| csv::Writer::from_path(p).unwrap());

    loop {
        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f64();
        last = now;

        let i: f64 = query(&mut stream, "MEAS:CURR?")
            .parse()
            .unwrap_or(0.0);

        // Discharge / charge integration
        soc -= i * dt / (profile.capacity_ah * 3600.0);
        soc = soc.clamp(0.0, 1.0);

        let voc = interpolate_ocv(&profile.ocv_curve, soc);

        // RC smoothing
        let tau = profile.rc_time_constant_ms as f64 / 1000.0;
        let alpha = dt / (tau + dt);

        let v_target = voc - i * profile.internal_resistance_ohm;
        v_filt += alpha * (v_target - v_filt);

        if v_filt <= profile.cutoff_voltage {
            println!("Cutoff reached");
            send(&mut stream, "OUTP OFF");
            break;
        }

        if v_filt >= profile.max_voltage {
            v_filt = profile.max_voltage;
        }

        send(&mut stream, &format!("VOLT {:.3}", v_filt));

        if let Some(w) = csv.as_mut() {
            w.write_record(&[
                format!("{:.3}", now.elapsed().as_secs_f64()),
                format!("{:.4}", soc),
                format!("{:.3}", v_filt),
                format!("{:.3}", i),
                format!("{:.3}", v_filt * i),
            ])
            .unwrap();
            w.flush().unwrap();
        }

        println!(
            "SoC={:>5.1}%  V={:.3} V  I={:.3} A  P={:.2} W",
            soc * 100.0,
            v_filt,
            i,
            v_filt * i
        );

        sleep(Duration::from_millis(profile.update_interval_ms));
    }
}
