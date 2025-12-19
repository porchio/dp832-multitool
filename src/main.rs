mod ui;

use clap::Parser;
use serde::Deserialize;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};

// Macro to log to UI only (no console output that messes up TUI)
macro_rules! log_message {
    ($state:expr, $writers:expr, $($arg:tt)*) => {{
        let msg = format!($($arg)*);
        if let Ok(mut s) = $state.lock() {
            s.add_log(msg.clone());
        }
        if let Ok(mut w) = $writers.lock() {
            w.write_event(&msg);
        }
    }};
}

// Macro to log SCPI commands to separate SCPI log
macro_rules! log_scpi {
    ($state:expr, $writers:expr, $($arg:tt)*) => {{
        let msg = format!($($arg)*);
        if let Ok(mut s) = $state.lock() {
            s.add_scpi_log(msg.clone());
        }
        if let Ok(mut w) = $writers.lock() {
            w.write_scpi(&msg);
        }
    }};
}

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

    /// Battery profile JSON files (can specify multiple, e.g., -p ch1.json -p ch2.json)
    #[arg(short, long)]
    profile: Vec<String>,

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

struct ScpiConnection {
    stream: TcpStream,
    selected_channel: Option<u8>,
    state: Arc<Mutex<ui::RuntimeState>>,
    writers: Arc<Mutex<ui::LogWriters>>,
    verbose_scpi: bool,
}

impl ScpiConnection {
    fn new(stream: TcpStream, state: Arc<Mutex<ui::RuntimeState>>, writers: Arc<Mutex<ui::LogWriters>>) -> Self {
        // Check if verbose SCPI logging is enabled
        let verbose_scpi = std::env::var("VERBOSE_SCPI").is_ok();
        
        Self {
            stream,
            selected_channel: None,
            state,
            writers,
            verbose_scpi,
        }
    }

    fn select_channel(&mut self, channel: u8) {
        if self.selected_channel != Some(channel) {
            let cmd = format!("INST:NSEL {}", channel);
            // Always log channel selection
            log_scpi!(self.state, self.writers, "→ {}", cmd);
            send(&mut self.stream, &cmd);
            self.selected_channel = Some(channel);
        }
    }

    fn send(&mut self, cmd: &str) {
        // Log important commands always, others only if verbose
        let is_important = cmd.starts_with("OUTP") || 
                          cmd.starts_with("VOLT ") ||
                          cmd.starts_with("CURR ") ||
                          cmd.starts_with("*");
        
        if is_important || self.verbose_scpi {
            log_scpi!(self.state, self.writers, "→ {}", cmd);
        }
        send(&mut self.stream, cmd);
    }

    fn query(&mut self, cmd: &str) -> String {
        // Log important queries always, others only if verbose
        let is_important = cmd == "*IDN?" || 
                          cmd.starts_with("MEAS:") ||
                          cmd.starts_with("SYST") ||
                          cmd.starts_with("OUTP?");
        
        if is_important || self.verbose_scpi {
            log_scpi!(self.state, self.writers, "→ {}", cmd);
        }
        let response = query(&mut self.stream, cmd);
        if is_important || self.verbose_scpi {
            log_scpi!(self.state, self.writers, "← {}", response.trim());
        }
        response
    }
}

fn send(stream: &mut TcpStream, cmd: &str) {
    let cmd = format!("{}\n", cmd);
    stream.write_all(cmd.as_bytes()).unwrap();
    stream.flush().unwrap();  // Ensure data is sent immediately
}

fn drain_buffer(stream: &mut TcpStream) {
    // Drain any leftover data in the buffer to prevent response bleed
    let mut buf = [0u8; 256];
    let timeout = std::time::Duration::from_millis(100);
    let start = std::time::Instant::now();
    
    while start.elapsed() < timeout {
        match stream.read(&mut buf) {
            Ok(0) => break,  // Connection closed
            Ok(_) => continue,  // Keep draining
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,  // No more data
            Err(_) => break,
        }
    }
}

fn query(stream: &mut TcpStream, cmd: &str) -> String {
    send(stream, cmd);
    
    // Delay to let device process command
    // Longer delay for *IDN? as it returns more data
    let delay = if cmd.starts_with("*IDN") {
        std::time::Duration::from_millis(100)
    } else {
        std::time::Duration::from_millis(50)
    };
    std::thread::sleep(delay);
    
    let mut resp = Vec::new();
    let mut buf = [0u8; 256];
    let start = std::time::Instant::now();
    
    // Longer timeout for *IDN? queries
    let timeout = if cmd.starts_with("*IDN") {
        std::time::Duration::from_millis(500)
    } else {
        std::time::Duration::from_millis(300)
    };

    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,  // Connection closed
            Ok(n) => {
                resp.extend_from_slice(&buf[..n]);
                if resp.ends_with(b"\n") {
                    break;  // Got complete response
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No data available yet, check timeout
                if start.elapsed() >= timeout {
                    break;  // Timeout reached
                }
                // Wait a bit and retry
                std::thread::sleep(std::time::Duration::from_millis(50));
                continue;
            }
            Err(e) => panic!("TCP read error: {}", e),
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

    // Resolve battery profiles
    let mut profile_paths = args.profile;
    if profile_paths.is_empty() {
        if let Some(battery_cfg) = cfg.battery {
            profile_paths.push(battery_cfg.profile);
        }
    }

    if profile_paths.is_empty() {
        eprintln!("Error: No battery profile specified");
        eprintln!("Use: -p <profile.json> (can specify multiple times for multiple channels)");
        std::process::exit(1);
    }

    // Load all profiles
    let mut profiles = Vec::new();
    for profile_path in &profile_paths {
        let mut json = String::new();
        File::open(profile_path)
            .unwrap_or_else(|e| {
                eprintln!("Failed to open profile {}: {}", profile_path, e);
                std::process::exit(1);
            })
            .read_to_string(&mut json)
            .unwrap();

        let profile: BatteryProfile = serde_json::from_str(&json)
            .unwrap_or_else(|e| {
                eprintln!("Failed to parse profile {}: {}", profile_path, e);
                std::process::exit(1);
            });
        
        println!("Loaded profile '{}' for channel {}", profile.name, profile.channel);
        profiles.push(profile);
    }

    // Resolve CSV log
    let csv_log = args.log.or_else(|| cfg.logging.and_then(|l| l.csv));

    println!("DP832: {}:{}", ip, port);
    println!("Active channels: {}", profiles.len());

    let addr = format!("{}:{}", ip, port);
    let stream = TcpStream::connect(&addr).unwrap();

    // Set to non-blocking mode with manual timeout handling
    stream.set_nonblocking(true).unwrap();

    // Initialize shared state
    let state = Arc::new(Mutex::new(ui::RuntimeState {
        channels: Default::default(),
        running: true,
        log_messages: Default::default(),
        scpi_log_messages: Default::default(),
    }));

    // Initialize log writers
    let writers = Arc::new(Mutex::new(ui::LogWriters::new()));

    // Create SCPI connection early (with logging support)
    let scpi_conn = ScpiConnection::new(stream, state.clone(), writers.clone());
    let mut conn = scpi_conn;
    
    // Clear errors and get ID (now with logging)
    conn.send("*CLS");
    std::thread::sleep(std::time::Duration::from_millis(50));
    
    let idn = conn.query("*IDN?");
    println!("{}", idn);
    
    // Drain buffer and add delay after *IDN? to prevent response bleed
    std::thread::sleep(std::time::Duration::from_millis(100));
    drain_buffer(&mut conn.stream);

    // Set up each channel
    for profile in &profiles {
        let ch_idx = (profile.channel - 1) as usize;
        if ch_idx < 3 {
            let mut s = state.lock().unwrap();
            s.channels[ch_idx].enabled = true;
            s.channels[ch_idx].soc = 1.0;
            s.channels[ch_idx].profile_name = profile.name.clone();
        }
    }

    // Share SCPI connection with channel tracking (mutex-protected)
    let shared_conn = Arc::new(Mutex::new(conn));
    
    // Start simulation threads for each channel
    for profile in profiles {
        let state_clone = state.clone();
        let writers_clone = writers.clone();
        let conn_clone = shared_conn.clone();
        
        let csv_clone = csv_log.as_ref().map(|p| {
            let path = format!("{}_ch{}.csv", p.trim_end_matches(".csv"), profile.channel);
            csv::Writer::from_path(path).unwrap()
        });

        std::thread::spawn(move || {
            simulate_channel(state_clone, writers_clone, conn_clone, profile, csv_clone);
        });
    }

    // Start TUI (blocking - runs until user quits)
    ui::run_tui(state.clone(), addr.clone());
}

fn simulate_channel(
    state: Arc<Mutex<ui::RuntimeState>>,
    writers: Arc<Mutex<ui::LogWriters>>,
    conn: Arc<Mutex<ScpiConnection>>,
    profile: BatteryProfile,
    mut csv: Option<csv::Writer<File>>,
) {
    let ch_idx = (profile.channel - 1) as usize;
    
    // Initialize channel
    {
        let mut c = conn.lock().unwrap();
        c.select_channel(profile.channel);
        
        // Turn off output (use channel-specific command)
        c.send(&format!("OUTP CH{},OFF", profile.channel));
        std::thread::sleep(std::time::Duration::from_millis(200));
        
        // Set current limit
        c.send(&format!("CURR {:.3}", profile.current_limit_discharge_a));
        std::thread::sleep(std::time::Duration::from_millis(200));
        
        // Turn on output
        c.send(&format!("OUTP CH{},ON", profile.channel));
        std::thread::sleep(std::time::Duration::from_millis(200));
        
        log_message!(state, writers, "CH{}: Initialized - {} ({:.1}Ah, {:.3}Ω)", 
                    profile.channel, 
                    profile.name,
                    profile.capacity_ah,
                    profile.internal_resistance_ohm);
    }

    let mut soc = 1.0;
    let mut last = Instant::now();
    let mut v_filt = interpolate_ocv(&profile.ocv_curve, soc);
    let mut consecutive_errors = 0;
    const MAX_CONSECUTIVE_ERRORS: u32 = 5;

    loop {
        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f64();
        last = now;

        // Query current directly without switching channel
        let curr_result: Result<f64, String> = {
            let mut c = conn.lock().unwrap();
            let curr_str = c.query(&format!("MEAS:CURR? CH{}", profile.channel));
            curr_str.trim().parse().map_err(|_| curr_str.clone())
        };

        // Handle parsing failure with retry logic
        let i = match curr_result {
            Ok(current) => {
                consecutive_errors = 0;  // Reset error counter on success
                if current.abs() > 0.001 {
                    log_message!(state, writers, "CH{}: Current = {:.3} A", profile.channel, current);
                }
                current
            }
            Err(raw_response) => {
                consecutive_errors += 1;
                log_message!(state, writers, "CH{}: ERROR #{} - Failed to parse current '{}'. Retrying...", 
                            profile.channel, consecutive_errors, raw_response.trim());
                
                if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                    log_message!(state, writers, "CH{}: Too many consecutive errors. Stopping simulation for safety.", 
                                profile.channel);
                    // Turn off output for safety
                    let mut c = conn.lock().unwrap();
                    c.select_channel(profile.channel);
                    c.send(&format!("OUTP CH{},OFF", profile.channel));
                    break;
                }
                
                // Skip this iteration and retry next time
                sleep(Duration::from_millis(profile.update_interval_ms));
                continue;
            }
        };

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
            log_message!(state, writers, "CH{}: Cutoff voltage reached ({:.3}V)", profile.channel, v_filt);
            let mut c = conn.lock().unwrap();
            c.select_channel(profile.channel);
            c.send(&format!("OUTP CH{},OFF", profile.channel));
            break;
        }

        if v_filt >= profile.max_voltage {
            v_filt = profile.max_voltage;
        }

        // Set voltage - requires channel selection
        {
            let mut c = conn.lock().unwrap();
            c.select_channel(profile.channel);  // Only switches if different
            c.send(&format!("VOLT {:.3}", v_filt));
            
            // Debug: verify voltage was set and measure actual output (commented for cleaner output)
            // let actual_v = c.query(&format!("MEAS:VOLT? CH{}", profile.channel));
            // let actual_i = c.query(&format!("MEAS:CURR? CH{}", profile.channel));
            // if now.elapsed().as_secs() % 5 == 0 {
            //     println!("CH{}: Set={:.3}V Measured={} Current={}", 
            //         profile.channel, v_filt, actual_v.trim(), actual_i.trim());
            // }
        }

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

        // Update shared state
        {
            let mut s = state.lock().unwrap();
            if ch_idx < 3 {
                s.channels[ch_idx].soc = soc;
                s.channels[ch_idx].voltage = v_filt;
                s.channels[ch_idx].current = i;
                s.channels[ch_idx].power = v_filt * i;
                s.channels[ch_idx].ocv = voc;
            }
        }

        if !state.lock().unwrap().running {
            let mut c = conn.lock().unwrap();
            c.select_channel(profile.channel);
            c.send(&format!("OUTP CH{},OFF", profile.channel));
            break;
        }

        sleep(Duration::from_millis(profile.update_interval_ms));
    }
    
    log_message!(state, writers, "CH{}: Simulation stopped", profile.channel);
}
