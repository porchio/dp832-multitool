/// DP832 Battery Simulator
/// 
/// Simulates realistic battery behavior on the Rigol DP832 power supply

use clap::Parser;
use dp832_battery_sim::battery_sim::{BatteryProfile, Config, interpolate_ocv};
use dp832_battery_sim::common::{LogWriters, RuntimeState};
use dp832_battery_sim::scpi::{send, query};
use std::fs::File;
use std::io::Read;
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
#[command(name = "dp832-battery-sim")]
#[command(about = "Battery simulator for Rigol DP832 power supply")]
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

fn main() {
    let args = Args::parse();

    let cfg: Config = dp832_battery_sim::common::load_optional_config(args.config.as_deref());
    
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
    let mut stream = TcpStream::connect(&addr).unwrap();

    // Set blocking mode with 1 second read timeout (as in working version)
    stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();

    // Clear errors and get ID
    send(&mut stream, "*CLS");
    println!("{}", query(&mut stream, "*IDN?"));

    // Initialize shared state
    let state = Arc::new(Mutex::new(RuntimeState {
        channels: Default::default(),
        running: true,
        log_messages: Default::default(),
        scpi_log_messages: Default::default(),
    }));

    // Initialize log writers
    let writers = Arc::new(Mutex::new(LogWriters::new()));

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

    // Start TUI in separate thread
    let tui_state = state.clone();
    let addr_clone = addr.clone();
    std::thread::spawn(move || {
        dp832_battery_sim::battery_sim::ui::run_tui(tui_state, addr_clone);
    });

    // Start simulation threads for each channel
    // Each channel gets its own TCP connection to avoid race conditions
    let mut sim_threads = Vec::new();
    
    for profile in profiles {
        let state_clone = state.clone();
        let writers_clone = writers.clone();
        
        // Create separate TCP stream for this channel (key to avoiding Command errors!)
        let mut stream_clone = TcpStream::connect(&addr).unwrap();
        stream_clone
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        
        // Clear any errors on this connection before starting
        send(&mut stream_clone, "*CLS");
        
        let csv_clone = csv_log.as_ref().map(|p| {
            let path = format!("{}_ch{}.csv", p.trim_end_matches(".csv"), profile.channel);
            csv::Writer::from_path(path).unwrap()
        });

        let thread = std::thread::spawn(move || {
            simulate_channel(state_clone, writers_clone, stream_clone, profile, csv_clone);
        });
        
        sim_threads.push(thread);
    }

    // Wait for all simulation threads to complete
    for thread in sim_threads {
        thread.join().unwrap();
    }
}

fn simulate_channel(
    state: Arc<Mutex<RuntimeState>>,
    writers: Arc<Mutex<LogWriters>>,
    mut stream: TcpStream,
    profile: BatteryProfile,
    mut csv: Option<csv::Writer<File>>,
) {
    let ch_idx = (profile.channel - 1) as usize;
    let ch_name = format!("CH{}", profile.channel);
    
    // Initialize channel - select it once at the start
    // Since each channel has its own TCP connection, this selection persists
    log_scpi!(state, writers, "CH{} → INST:NSEL {}", profile.channel, profile.channel);
    send(&mut stream, &format!("INST:NSEL {}", profile.channel));
    
    log_scpi!(state, writers, "CH{} → OUTP OFF", profile.channel);
    send(&mut stream, "OUTP OFF");
    
    log_scpi!(state, writers, "CH{} → CURR {:.3}", profile.channel, profile.current_limit_discharge_a);
    send(&mut stream, &format!("CURR {:.3}", profile.current_limit_discharge_a));
    
    log_scpi!(state, writers, "CH{} → OUTP ON", profile.channel);
    send(&mut stream, "OUTP ON");
    
    log_message!(state, writers, "CH{}: Initialized - {} ({:.1}Ah, {:.3}Ω)", 
                profile.channel, 
                profile.name,
                profile.capacity_ah,
                profile.internal_resistance_ohm);

    let mut soc = 1.0;
    let mut last = Instant::now();
    let mut v_filt = interpolate_ocv(&profile.ocv_curve, soc);
    let mut last_voltage_set = v_filt;  // Track last voltage we sent to PSU
    let mut consecutive_errors = 0;
    const MAX_CONSECUTIVE_ERRORS: u32 = 5;
    const VOLTAGE_CHANGE_THRESHOLD: f64 = 0.001;  // Only update if voltage changes by >1mV

    loop {
        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f64();
        last = now;

        // Query current using channel-specific syntax (more reliable than relying on INST:NSEL)
        let curr_cmd = format!("MEAS:CURR? {}", ch_name);
        log_scpi!(state, writers, "{} → {}", ch_name, curr_cmd);
        let curr_str = query(&mut stream, &curr_cmd);
        log_scpi!(state, writers, "{} ← {}", ch_name, curr_str.trim());
        
        // Check for error responses before parsing
        let curr_result: Result<f64, String> = {
            let trimmed = curr_str.trim();
            if trimmed.contains("error") || trimmed.contains("Error") || trimmed.contains("ERROR") {
                // PSU returned error - clear it and retry
                log_message!(state, writers, "CH{}: PSU error response '{}' - clearing error state", 
                            profile.channel, trimmed);
                send(&mut stream, "*CLS");  // Clear error state
                Err(trimmed.to_string())
            } else {
                trimmed.parse().map_err(|_| trimmed.to_string())
            }
        };

        // Handle parsing failure with retry logic
        let i = match curr_result {
            Ok(current) => {
                consecutive_errors = 0;  // Reset error counter on success
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
                    log_scpi!(state, writers, "CH{} → OUTP OFF", profile.channel);
                    send(&mut stream, "OUTP OFF");
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
            log_scpi!(state, writers, "CH{} → OUTP OFF", profile.channel);
            send(&mut stream, "OUTP OFF");
            break;
        }

        if v_filt >= profile.max_voltage {
            v_filt = profile.max_voltage;
        }

        // Set voltage - only if it has changed significantly (reduces SCPI traffic)
        // No need to re-select channel since it was selected at init and persists on this connection
        if (v_filt - last_voltage_set).abs() > VOLTAGE_CHANGE_THRESHOLD {
            let volt_cmd = format!("VOLT {:.3}", v_filt);
            log_scpi!(state, writers, "{} → {}", ch_name, volt_cmd);
            send(&mut stream, &volt_cmd);
            
            last_voltage_set = v_filt;
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
            log_scpi!(state, writers, "CH{} → OUTP OFF", profile.channel);
            send(&mut stream, "OUTP OFF");
            break;
        }

        sleep(Duration::from_millis(profile.update_interval_ms));
    }
    
    log_message!(state, writers, "CH{}: Simulation stopped", profile.channel);
}
