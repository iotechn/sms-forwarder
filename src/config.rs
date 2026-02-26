use serde::{Deserialize, Serialize};
use std::fs;

fn default_modem_device() -> String {
    "/dev/ttyUSB2".to_string()
}
fn default_baud_rate() -> u32 {
    115200
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub bark_key: String,
    pub aes_key: String,
    pub aes_iv: String,
    /// 短信内容包含任一关键字时，以紧急通知推送
    pub emergency_keywords: Vec<String>,
    /// 调制解调器串口设备路径（AT 指令直连）
    #[serde(default = "default_modem_device")]
    pub modem_device: String,
    /// 串口波特率
    #[serde(default = "default_baud_rate")]
    pub baud_rate: u32,
}

impl Config {
    pub fn load() -> Self {
        tracing::debug!("reading config.json");
        let txt = fs::read_to_string("config.json").unwrap();
        let cfg = serde_json::from_str(&txt).unwrap();
        tracing::debug!("config parsed");
        cfg
    }

    pub fn save(&self) {
        tracing::info!("saving config to config.json");
        fs::write(
            "config.json",
            serde_json::to_string_pretty(self).unwrap(),
        ).unwrap();
    }
}
