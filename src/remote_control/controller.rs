// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (C) 2025 Marcus Folkesson

/// DP832 Controller
/// 
/// Manages communication and control of the DP832 power supply

use std::net::TcpStream;
use std::time::Duration;
use std::sync::mpsc::Sender;
use crate::scpi::{send, query};

pub struct DP832Controller {
    stream: TcpStream,
    pub channels: [ChannelState; 3],
    pub device_id: String,
    scpi_logger: Option<Sender<String>>,
}

#[derive(Clone)]
pub struct ChannelState {
    pub voltage_set: f64,
    pub current_set: f64,
    pub voltage_actual: f64,
    pub current_actual: f64,
    pub power_actual: f64,
    pub enabled: bool,
}

impl Default for ChannelState {
    fn default() -> Self {
        Self {
            voltage_set: 0.0,
            current_set: 0.0,
            voltage_actual: 0.0,
            current_actual: 0.0,
            power_actual: 0.0,
            enabled: false,
        }
    }
}

impl DP832Controller {
    /// Create a new controller and connect to the device
    pub fn new(addr: &str) -> Result<Self, std::io::Error> {
        let mut stream = TcpStream::connect(addr)?;
        stream.set_read_timeout(Some(Duration::from_secs(1)))?;
        
        // Initialize connection
        send(&mut stream, "*CLS");
        let device_id = query(&mut stream, "*IDN?");
        
        let mut controller = Self {
            stream,
            channels: Default::default(),
            device_id,
            scpi_logger: None,
        };
        
        // Read initial state
        controller.update_all_channels()?;
        
        Ok(controller)
    }
    
    /// Set SCPI logger sender
    pub fn set_scpi_logger(&mut self, sender: Sender<String>) {
        self.scpi_logger = Some(sender);
    }
    
    /// Log SCPI command
    fn log_scpi(&mut self, cmd: &str) {
        if let Some(ref sender) = self.scpi_logger {
            let _ = sender.send(cmd.to_string());
        }
    }
    
    /// Update measurements for all channels
    pub fn update_all_channels(&mut self) -> Result<(), std::io::Error> {
        for ch in 1..=3 {
            self.update_channel(ch)?;
        }
        Ok(())
    }
    
    /// Update measurements for a specific channel
    /// This function does NOT switch the active channel on the PSU
    pub fn update_channel(&mut self, channel: u8) -> Result<(), std::io::Error> {
        let ch_idx = (channel - 1) as usize;
        if ch_idx >= 3 {
            return Ok(());
        }
        
        let ch_name = format!("CH{}", channel);
        
        // Read actual voltage (no channel switch needed)
        let cmd = format!("MEAS:VOLT? {}", ch_name);
        self.log_scpi(&cmd);
        let v_act_str = query(&mut self.stream, &cmd);
        if let Ok(v) = v_act_str.trim().parse::<f64>() {
            self.channels[ch_idx].voltage_actual = v;
        }
        
        // Read actual current (no channel switch needed)
        let cmd = format!("MEAS:CURR? {}", ch_name);
        self.log_scpi(&cmd);
        let i_act_str = query(&mut self.stream, &cmd);
        if let Ok(i) = i_act_str.trim().parse::<f64>() {
            self.channels[ch_idx].current_actual = i;
        }
        
        // Calculate power
        self.channels[ch_idx].power_actual = 
            self.channels[ch_idx].voltage_actual * self.channels[ch_idx].current_actual;
        
        // Read output state (no channel switch needed)
        let cmd = format!("OUTP? {}", ch_name);
        self.log_scpi(&cmd);
        let out_str = query(&mut self.stream, &cmd);
        self.channels[ch_idx].enabled = out_str.trim() == "ON";
        
        // Read voltage and current setpoints using APPL? command
        // This avoids switching the active channel on the PSU
        // APPL? returns format: "CH1,3.300,2.000,ON" or similar
        let cmd = format!("APPL? {}", ch_name);
        self.log_scpi(&cmd);
        let appl_str = query(&mut self.stream, &cmd);
        let parts: Vec<&str> = appl_str.split(',').collect();
        if parts.len() >= 3 {
            if let Ok(v) = parts[1].trim().parse::<f64>() {
                self.channels[ch_idx].voltage_set = v;
            }
            if let Ok(i) = parts[2].trim().parse::<f64>() {
                self.channels[ch_idx].current_set = i;
            }
        }
        
        Ok(())
    }
    
    /// Set voltage for a channel
    pub fn set_voltage(&mut self, channel: u8, voltage: f64) -> Result<(), std::io::Error> {
        if channel < 1 || channel > 3 {
            return Ok(());
        }
        
        // Use APPL command to set voltage without switching channel
        // APPL CH1,<voltage>,<current>
        let ch_idx = (channel - 1) as usize;
        let current = self.channels[ch_idx].current_set;
        let cmd = format!("APPL CH{},{:.3},{:.3}", channel, voltage, current);
        self.log_scpi(&cmd);
        send(&mut self.stream, &cmd);
        
        self.channels[ch_idx].voltage_set = voltage;
        
        Ok(())
    }
    
    /// Set current for a channel
    pub fn set_current(&mut self, channel: u8, current: f64) -> Result<(), std::io::Error> {
        if channel < 1 || channel > 3 {
            return Ok(());
        }
        
        // Use APPL command to set current without switching channel
        // APPL CH1,<voltage>,<current>
        let ch_idx = (channel - 1) as usize;
        let voltage = self.channels[ch_idx].voltage_set;
        let cmd = format!("APPL CH{},{:.3},{:.3}", channel, voltage, current);
        self.log_scpi(&cmd);
        send(&mut self.stream, &cmd);
        
        self.channels[ch_idx].current_set = current;
        
        Ok(())
    }
    
    /// Enable or disable a channel
    pub fn set_output(&mut self, channel: u8, enabled: bool) -> Result<(), std::io::Error> {
        if channel < 1 || channel > 3 {
            return Ok(());
        }
        
        let state = if enabled { "ON" } else { "OFF" };
        let cmd = format!("OUTP CH{},{}", channel, state);
        self.log_scpi(&cmd);
        send(&mut self.stream, &cmd);
        
        let ch_idx = (channel - 1) as usize;
        self.channels[ch_idx].enabled = enabled;
        
        Ok(())
    }
    
    /// Enable all channels at once
    pub fn enable_all_channels(&mut self) -> Result<(), std::io::Error> {
        let cmd = "OUTP ALL,ON";
        self.log_scpi(cmd);
        send(&mut self.stream, cmd);
        
        // Update all channel states
        for ch in 0..3 {
            self.channels[ch].enabled = true;
        }
        
        Ok(())
    }
    
    /// Disable all channels at once
    pub fn disable_all_channels(&mut self) -> Result<(), std::io::Error> {
        let cmd = "OUTP ALL,OFF";
        self.log_scpi(cmd);
        send(&mut self.stream, cmd);
        
        // Update all channel states
        for ch in 0..3 {
            self.channels[ch].enabled = false;
        }
        
        Ok(())
    }
    
    /// Get device identification
    pub fn get_device_id(&self) -> &str {
        &self.device_id
    }
}
