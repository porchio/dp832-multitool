/// Remote control configuration

use serde::Deserialize;
use crate::common::DeviceConfig;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub device: Option<DeviceConfig>,
}
