// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (C) 2025 Marcus Folkesson

/// Remote control configuration

use serde::Deserialize;
use crate::common::DeviceConfig;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub device: Option<DeviceConfig>,
}
