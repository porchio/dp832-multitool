// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2024 Marcus Hoffmann

/// Common utilities and types shared across modules

use serde::Deserialize;
use std::fs::File;
use std::io::{Read, Write};
use std::collections::VecDeque;

/// Device configuration
#[derive(Debug, Deserialize)]
pub struct DeviceConfig {
    pub ip: String,
    pub port: Option<u16>,
}

/// Channel state for UI display
#[derive(Clone, Default)]
pub struct ChannelState {
    pub soc: f64,
    pub voltage: f64,
    pub current: f64,
    pub power: f64,
    pub ocv: f64,
    pub profile_name: String,
    pub enabled: bool,
}

/// Log file writers for event and SCPI logs
pub struct LogWriters {
    event_log: Option<File>,
    scpi_log: Option<File>,
}

impl LogWriters {
    pub fn new() -> Self {
        // Create logs directory if it doesn't exist
        let _ = std::fs::create_dir_all("logs");
        
        // Create timestamped log files
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        
        let event_log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(format!("logs/event_{}.log", timestamp))
            .ok();
            
        let scpi_log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(format!("logs/scpi_{}.log", timestamp))
            .ok();
        
        Self {
            event_log,
            scpi_log,
        }
    }
    
    pub fn write_event(&mut self, message: &str) {
        if let Some(ref mut f) = self.event_log {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let _ = writeln!(f, "[{}] {}", timestamp, message);
            let _ = f.flush();
        }
    }
    
    pub fn write_scpi(&mut self, message: &str) {
        if let Some(ref mut f) = self.scpi_log {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let _ = writeln!(f, "[{}] {}", timestamp, message);
            let _ = f.flush();
        }
    }
}

/// Runtime state for UI
#[derive(Clone, Default)]
pub struct RuntimeState {
    pub channels: [ChannelState; 3],
    pub running: bool,
    pub log_messages: VecDeque<String>,
    pub scpi_log_messages: VecDeque<String>,
}

impl RuntimeState {
    pub fn add_log(&mut self, message: String) {
        self.log_messages.push_back(message);
        // Keep last 100 messages
        if self.log_messages.len() > 100 {
            self.log_messages.pop_front();
        }
    }
    
    pub fn add_scpi_log(&mut self, message: String) {
        self.scpi_log_messages.push_back(message);
        // Keep last 200 SCPI messages (more detailed)
        if self.scpi_log_messages.len() > 200 {
            self.scpi_log_messages.pop_front();
        }
    }
}

/// Load optional configuration file
pub fn load_optional_config<T: for<'de> Deserialize<'de> + Default>(path: Option<&str>) -> T {
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
            T::default()
        }
    } else {
        T::default()
    }
}

fn default_config_path() -> Option<std::path::PathBuf> {
    let base = dirs_next::config_dir()?;
    Some(base.join("dp832-battery").join("config.toml"))
}
