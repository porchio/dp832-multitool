// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (C) 2025 Marcus Folkesson

/// Remote Control Module for DP832
/// 
/// Provides a complete remote interface for controlling the DP832 power supply

pub mod ui;
pub mod config;
pub mod controller;

pub use config::*;
pub use controller::*;
