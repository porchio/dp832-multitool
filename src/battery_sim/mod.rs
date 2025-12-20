// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (C) 2025 Marcus Folkesson

/// Battery Simulator Module
/// 
/// Simulates realistic battery behavior on the DP832 power supply

pub mod model;
pub mod config;
pub mod ui;

pub use model::*;
pub use config::*;
