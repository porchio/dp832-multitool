/// DP832 Controller
/// 
/// Manages communication and control of the DP832 power supply

use std::net::TcpStream;
use std::time::Duration;
use crate::scpi::{send, query};

pub struct DP832Controller {
    stream: TcpStream,
    pub channels: [ChannelState; 3],
    pub device_id: String,
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
        };
        
        // Read initial state
        controller.update_all_channels()?;
        
        Ok(controller)
    }
    
    /// Update measurements for all channels
    pub fn update_all_channels(&mut self) -> Result<(), std::io::Error> {
        for ch in 1..=3 {
            self.update_channel(ch)?;
        }
        Ok(())
    }
    
    /// Update measurements for a specific channel
    pub fn update_channel(&mut self, channel: u8) -> Result<(), std::io::Error> {
        let ch_idx = (channel - 1) as usize;
        if ch_idx >= 3 {
            return Ok(());
        }
        
        let ch_name = format!("CH{}", channel);
        
        // Read voltage setpoint
        send(&mut self.stream, &format!("INST:NSEL {}", channel));
        let v_set_str = query(&mut self.stream, "VOLT?");
        if let Ok(v) = v_set_str.trim().parse::<f64>() {
            self.channels[ch_idx].voltage_set = v;
        }
        
        // Read current setpoint
        let i_set_str = query(&mut self.stream, "CURR?");
        if let Ok(i) = i_set_str.trim().parse::<f64>() {
            self.channels[ch_idx].current_set = i;
        }
        
        // Read actual voltage
        let v_act_str = query(&mut self.stream, &format!("MEAS:VOLT? {}", ch_name));
        if let Ok(v) = v_act_str.trim().parse::<f64>() {
            self.channels[ch_idx].voltage_actual = v;
        }
        
        // Read actual current
        let i_act_str = query(&mut self.stream, &format!("MEAS:CURR? {}", ch_name));
        if let Ok(i) = i_act_str.trim().parse::<f64>() {
            self.channels[ch_idx].current_actual = i;
        }
        
        // Calculate power
        self.channels[ch_idx].power_actual = 
            self.channels[ch_idx].voltage_actual * self.channels[ch_idx].current_actual;
        
        // Read output state
        let out_str = query(&mut self.stream, &format!("OUTP? {}", ch_name));
        self.channels[ch_idx].enabled = out_str.trim() == "ON";
        
        Ok(())
    }
    
    /// Set voltage for a channel
    pub fn set_voltage(&mut self, channel: u8, voltage: f64) -> Result<(), std::io::Error> {
        if channel < 1 || channel > 3 {
            return Ok(());
        }
        
        send(&mut self.stream, &format!("INST:NSEL {}", channel));
        send(&mut self.stream, &format!("VOLT {:.3}", voltage));
        
        let ch_idx = (channel - 1) as usize;
        self.channels[ch_idx].voltage_set = voltage;
        
        Ok(())
    }
    
    /// Set current for a channel
    pub fn set_current(&mut self, channel: u8, current: f64) -> Result<(), std::io::Error> {
        if channel < 1 || channel > 3 {
            return Ok(());
        }
        
        send(&mut self.stream, &format!("INST:NSEL {}", channel));
        send(&mut self.stream, &format!("CURR {:.3}", current));
        
        let ch_idx = (channel - 1) as usize;
        self.channels[ch_idx].current_set = current;
        
        Ok(())
    }
    
    /// Enable or disable a channel
    pub fn set_output(&mut self, channel: u8, enabled: bool) -> Result<(), std::io::Error> {
        if channel < 1 || channel > 3 {
            return Ok(());
        }
        
        let state = if enabled { "ON" } else { "OFF" };
        send(&mut self.stream, &format!("OUTP {},{}", state, format!("CH{}", channel)));
        
        let ch_idx = (channel - 1) as usize;
        self.channels[ch_idx].enabled = enabled;
        
        Ok(())
    }
    
    /// Get device identification
    pub fn get_device_id(&self) -> &str {
        &self.device_id
    }
}
