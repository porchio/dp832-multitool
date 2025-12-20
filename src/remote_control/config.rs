// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2024 Marcus Hoffmann

/// Remote control configuration

use serde::Deserialize;
use crate::common::DeviceConfig;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub device: Option<DeviceConfig>,
}
