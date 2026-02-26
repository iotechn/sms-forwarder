use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{sleep, timeout};
use tokio_serial::SerialStream;

use crate::bark;
use crate::config::Config;

const AT_TIMEOUT: Duration = Duration::from_secs(5);
const RESPONSE_READ_TIMEOUT: Duration = Duration::from_secs(10);

/// 打开串口
fn open_port(device: &str, baud_rate: u32) -> Result<SerialStream, tokio_serial::Error> {
    let builder = tokio_serial::new(device, baud_rate)
        .timeout(Duration::from_millis(500));
    SerialStream::open(&builder)
}

/// 发送 AT 指令并读取响应直到 OK/ERROR 或超时
async fn at_command(
    port: &mut SerialStream,
    cmd: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let line = format!("AT{}\r\n", cmd);
    port.write_all(line.as_bytes()).await?;
    port.flush().await?;

    let mut buf = Vec::with_capacity(1024);
    let mut read_buf = [0u8; 256];
    let deadline = tokio::time::Instant::now() + RESPONSE_READ_TIMEOUT;

    loop {
        if tokio::time::Instant::now() > deadline {
            break;
        }
        tokio::select! {
            r = port.read(&mut read_buf) => {
                let n = r?;
                if n == 0 {
                    break;
                }
                buf.extend_from_slice(&read_buf[..n]);
                let s = String::from_utf8_lossy(&buf);
                if s.contains("OK") || s.contains("ERROR") {
                    break;
                }
            }
            _ = sleep(Duration::from_millis(100)) => {}
        }
    }

    Ok(String::from_utf8_lossy(&buf).into_owned())
}

/// 发送短信（AT+CMGF=1 文本模式，AT+CMGS 后跟号码，再发正文 + Ctrl+Z）
pub async fn send_sms(cfg: &Config, number: &str, text: &str) {
    tracing::info!(number = %number, "sending SMS via AT");
    let mut port = match open_port(&cfg.modem_device, cfg.baud_rate) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(err = %e, "open modem port failed");
            return;
        }
    };

    let run = async {
        at_command(&mut port, "+CMGF=1").await?;
        let cmd = format!("+CMGS=\"{}\"", number);
        port.write_all(format!("AT{}\r\n", cmd).as_bytes()).await?;
        port.flush().await?;
        sleep(Duration::from_millis(300)).await;
        port.write_all(text.as_bytes()).await?;
        port.write_all(&[0x1A]).await?; // Ctrl+Z
        port.flush().await?;

        let mut buf = vec![0u8; 512];
        let mut total = 0usize;
        let deadline = tokio::time::Instant::now() + RESPONSE_READ_TIMEOUT;
        while tokio::time::Instant::now() < deadline && total < buf.len() {
            if let Ok(n) = timeout(Duration::from_millis(200), port.read(&mut buf[total..])).await {
                if let Ok(sz) = n {
                    total += sz;
                    if sz == 0 {
                        break;
                    }
                    let s = String::from_utf8_lossy(&buf[..total]);
                    if s.contains("OK") || s.contains("ERROR") || s.contains("+CMGS:") {
                        break;
                    }
                }
            }
        }
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    };

    match timeout(AT_TIMEOUT, run).await {
        Ok(Ok(())) => tracing::info!(number = %number, "发送成功"),
        Ok(Err(e)) => tracing::warn!(err = %e, "AT send_sms failed"),
        Err(_) => tracing::warn!("send_sms timeout"),
    }
}

/// 一条短信（索引、号码、正文）
struct SmsEntry {
    index: u32,
    number: String,
    body: String,
}

/// 解析 AT+CMGL 输出，提取 +CMGL: 行及后续正文
fn parse_cmgl_response(response: &str) -> Vec<SmsEntry> {
    let mut list = Vec::new();
    let mut lines = response.lines();
    let mut current_index = 0u32;
    let mut current_number = String::new();
    let mut current_body = String::new();
    let mut in_body = false;

    while let Some(line) = lines.next() {
        let line = line.trim();
        if line.starts_with("+CMGL:") {
            if in_body && !current_body.is_empty() {
                list.push(SmsEntry {
                    index: current_index,
                    number: current_number.clone(),
                    body: current_body.trim().to_string(),
                });
            }
            in_body = true;
            current_body.clear();
            // +CMGL: <index>,<stat>,"<number>","","<date>" 或类似
            let rest = line.strip_prefix("+CMGL:").unwrap_or("").trim();
            let parts: Vec<&str> = rest.split(',').map(|s| s.trim()).collect();
            if let Some(idx) = parts.get(0).and_then(|s| s.parse::<u32>().ok()) {
                current_index = idx;
            }
            if let Some(num) = parts.get(2) {
                current_number = num.trim_matches('"').to_string();
            }
        } else if in_body && !line.is_empty() && !line.eq_ignore_ascii_case("OK") {
            if !current_body.is_empty() {
                current_body.push('\n');
            }
            current_body.push_str(line);
        }
    }
    if in_body && !current_body.is_empty() {
        list.push(SmsEntry {
            index: current_index,
            number: current_number.clone(),
            body: current_body.trim().to_string(),
        });
    }
    list
}

pub async fn start(cfg: Config) {
    tracing::info!("SMS poll loop started (AT), interval 5s, device {}", cfg.modem_device);
    loop {
        let mut port = match open_port(&cfg.modem_device, cfg.baud_rate) {
            Ok(p) => p,
            Err(e) => {
                tracing::error!(err = %e, "open modem port failed");
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        let list_result = async {
            at_command(&mut port, "+CMGF=1").await?;
            let out = at_command(&mut port, "+CMGL=\"ALL\"").await?;
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(out)
        };

        let response = match timeout(AT_TIMEOUT, list_result).await {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                tracing::warn!(err = %e, "AT list SMS failed");
                sleep(Duration::from_secs(5)).await;
                continue;
            }
            Err(_) => {
                tracing::warn!("AT list SMS timeout");
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        let entries = parse_cmgl_response(&response);
        for ent in entries {
            tracing::info!(index = ent.index, number = %ent.number, "轮询到短信");
            handle_sms(&cfg, ent.number.clone(), ent.body.clone()).await;

            let idx = ent.index;
            let delete_result = async {
                at_command(&mut port, &format!("+CMGD={}", idx)).await
            };
            if let Ok(Ok(_)) = timeout(Duration::from_secs(2), delete_result).await {
                tracing::debug!(index = idx, "deleted");
            }
        }

        drop(port);
        sleep(Duration::from_secs(5)).await;
    }
}

fn decode_ucs2_hex(text: &str) -> Option<String> {
    let s = text.trim();
    if s.is_empty() || s.len() % 4 != 0 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }

    let mut bytes = Vec::with_capacity(s.len() / 2);
    for i in (0..s.len()).step_by(2) {
        let b = u8::from_str_radix(&s[i..i + 2], 16).ok()?;
        bytes.push(b);
    }
    if bytes.len() % 2 != 0 {
        return None;
    }

    let mut words = Vec::with_capacity(bytes.len() / 2);
    for i in (0..bytes.len()).step_by(2) {
        words.push(u16::from_be_bytes([bytes[i], bytes[i + 1]]));
    }

    String::from_utf16(&words).ok()
}

async fn handle_sms(cfg: &Config, number: String, text: String) {
    let decoded = decode_ucs2_hex(&text).unwrap_or_else(|| text.clone());

    let emergency = cfg
        .emergency_keywords
        .iter()
        .any(|kw| decoded.contains(kw));
    let title = format!("SMS from {}", number);

    if emergency {
        tracing::info!(number = %number, "emergency keyword matched, push as critical");
    }
    match bark::push(cfg, &title, &decoded, emergency).await {
        Ok(()) => tracing::info!(number = %number, "转发成功"),
        Err(e) => tracing::error!(number = %number, err = %e, "Bark push failed"),
    }
}
