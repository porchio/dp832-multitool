/// DP832 Remote Control Interface
/// 
/// Complete remote control interface for the Rigol DP832 power supply

use clap::Parser;
use dp832_battery_sim::remote_control::{Config, DP832Controller};
use dp832_battery_sim::remote_control::ui::RemoteControlUI;

#[derive(Parser)]
#[command(name = "dp832-remote")]
#[command(about = "Remote control interface for Rigol DP832 power supply")]
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

    let addr = format!("{}:{}", ip, port);
    
    println!("Connecting to DP832 at {}...", addr);
    
    let controller = DP832Controller::new(&addr)
        .unwrap_or_else(|e| {
            eprintln!("Failed to connect: {}", e);
            std::process::exit(1);
        });
    
    println!("Connected: {}", controller.get_device_id());
    println!("Starting remote control interface...");
    
    let mut ui = RemoteControlUI::new(controller);
    
    if let Err(e) = ui.run() {
        eprintln!("UI error: {}", e);
        std::process::exit(1);
    }
}
