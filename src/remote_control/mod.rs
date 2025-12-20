// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2024 Marcus Hoffmann

/// Remote Control Module for DP832
/// 
/// Provides a complete remote interface for controlling the DP832 power supply

pub mod ui;
pub mod config;
pub mod controller;

pub use config::*;
pub use controller::*;
