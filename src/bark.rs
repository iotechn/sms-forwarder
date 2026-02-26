// 加解密方式与 Bark 文档一致: https://bark.day.app/#/encryption
// 整包 JSON 使用 AES-128-CBC 加密，Base64 后以 POST 表单 ciphertext + iv 提交

use aes::Aes128;
use base64::{engine::general_purpose, Engine};
use cbc::Encryptor;
use cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::Client;
use serde::Serialize;

use crate::config::Config;

type Aes128CbcEnc = Encryptor<Aes128>;

/// 与 Bark 服务端约定的推送 payload 结构
#[derive(Serialize)]
struct BarkPayload {
    body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sound: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    volume: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    call: Option<u8>,
}

/// 使用 AES-128-CBC 加密明文，返回 Base64 密文（与 OpenSSL enc -aes-128-cbc 行为一致）
fn encrypt(plaintext: &str, key: &[u8], iv: &[u8]) -> String {
    let cipher = Aes128CbcEnc::new_from_slices(key, iv).expect("key/iv length");
    let mut buf = vec![0u8; (plaintext.len() / 16 + 1) * 16];
    buf[..plaintext.len()].copy_from_slice(plaintext.as_bytes());
    let ciphertext = cipher
        .encrypt_padded_mut::<Pkcs7>(&mut buf, plaintext.len())
        .expect("encrypt");
    general_purpose::STANDARD.encode(ciphertext)
}

pub async fn push(
    cfg: &Config,
    title: &str,
    body: &str,
    emergency: bool,
) -> anyhow::Result<()> {
    let key = cfg.aes_key.as_bytes();
    let iv = cfg.aes_iv.as_bytes();
    if key.len() != 16 || iv.len() != 16 {
        anyhow::bail!("aes_key 与 aes_iv 必须均为 16 字节");
    }

    let payload = BarkPayload {
        body: body.to_string(),
        title: if title.is_empty() {
            None
        } else {
            Some(title.to_string())
        },
        sound: Some("birdsong".to_string()),
        level: if emergency {
            Some("critical".to_string())
        } else {
            None
        },
        volume: if emergency { Some(5) } else { None },
        call: if emergency { Some(1) } else { None },
    };

    let json = serde_json::to_string(&payload)?;
    let ciphertext = encrypt(&json, key, iv);

    let url = format!("https://api.day.app/{}", cfg.bark_key);
    let body = format!(
        "ciphertext={}&iv={}",
        utf8_percent_encode(&ciphertext, NON_ALPHANUMERIC),
        utf8_percent_encode(&cfg.aes_iv, NON_ALPHANUMERIC)
    );

    tracing::debug!(title = %title, emergency = emergency, "pushing to Bark (encrypted)");
    let res = Client::new()
        .post(&url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await;
    match &res {
        Ok(r) => tracing::debug!(status = %r.status(), "Bark request completed"),
        Err(e) => tracing::error!(err = %e, "Bark request failed"),
    }
    res?;

    Ok(())
}
