// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2024 Marcus Hoffmann

/// Battery simulator configuration

use serde::Deserialize;
use crate::common::DeviceConfig;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub device: Option<DeviceConfig>,
    pub battery: Option<BatteryConfig>,
    pub logging: Option<LoggingConfig>,
}

#[derive(Debug, Deserialize)]
pub struct BatteryConfig {
    pub profile: String,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub csv: Option<String>,
}
