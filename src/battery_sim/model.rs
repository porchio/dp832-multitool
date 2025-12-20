// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (C) 2025 Marcus Folkesson

/// Battery model and simulation logic

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OcvPoint {
    pub soc: f64,
    pub voltage: f64,
}

#[derive(Debug, Deserialize)]
pub struct BatteryProfile {
    pub name: String,
    pub channel: u8,

    pub capacity_ah: f64,
    pub internal_resistance_ohm: f64,

    pub current_limit_discharge_a: f64,
    pub current_limit_charge_a: f64,

    pub cutoff_voltage: f64,
    pub max_voltage: f64,

    pub rc_time_constant_ms: u64,
    pub update_interval_ms: u64,

    pub ocv_curve: Vec<OcvPoint>,
}

/// Interpolate OCV from the OCV curve based on SoC
pub fn interpolate_ocv(curve: &[OcvPoint], soc: f64) -> f64 {
    let soc = soc.clamp(0.0, 1.0);

    for w in curve.windows(2) {
        if soc <= w[0].soc && soc >= w[1].soc {
            let t = (soc - w[1].soc) / (w[0].soc - w[1].soc);
            return w[1].voltage + t * (w[0].voltage - w[1].voltage);
        }
    }

    curve.last().unwrap().voltage
}
