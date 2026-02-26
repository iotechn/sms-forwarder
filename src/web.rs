use axum::{
    extract::State,
    response::Html,
    routing::{get, post},
    Json, Router,
};

use crate::config::Config;
use crate::modem;

pub fn router(cfg: Config) -> Router {
    Router::new()
        .route("/", get(index_page))
        .route("/settings", get(settings_page))
        .route("/config", get(get_cfg))
        .route("/config", post(set_cfg))
        .route("/send", post(send_sms))
        .with_state(cfg)
}

async fn index_page() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn settings_page() -> Html<&'static str> {
    Html(SETTINGS_HTML)
}

async fn get_cfg(State(cfg): State<Config>) -> Json<Config> {
    tracing::debug!("GET /config");
    Json(cfg)
}

async fn set_cfg(State(_cfg): State<Config>, Json(new): Json<Config>) {
    tracing::info!("POST /config, saving");
    new.save();
}

async fn send_sms(State(cfg): State<Config>, Json((number, text)): Json<(String, String)>) {
    tracing::info!(number = %number, "接收到发送的命令");
    modem::send_sms(&cfg, &number, &text).await;
}

// 简单双页面前端，黑白配色
const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <title>SMS Forwarder - 发送短信</title>
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <style>
    :root {
      color-scheme: light dark;
      --bg: #000;
      --fg: #fff;
      --fg-muted: #aaa;
      --border: #333;
      --accent: #fff;
      --accent-muted: #666;
      --error: #ff4d4f;
      --success: #52c41a;
    }
    * {
      box-sizing: border-box;
    }
    body {
      margin: 0;
      font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      background-color: var(--bg);
      color: var(--fg);
      min-height: 100vh;
      display: flex;
      align-items: center;
      justify-content: center;
    }
    .app {
      width: 100%;
      max-width: 520px;
      padding: 24px 20px 20px;
      border: 1px solid var(--border);
      border-radius: 16px;
      background: radial-gradient(circle at top left, #111 0, #000 40%);
      box-shadow: 0 18px 45px rgba(0, 0, 0, 0.7);
    }
    .header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      margin-bottom: 20px;
    }
    .title-group {
      display: flex;
      flex-direction: column;
      gap: 4px;
    }
    .title {
      font-size: 20px;
      font-weight: 600;
      letter-spacing: .04em;
      text-transform: uppercase;
    }
    .subtitle {
      font-size: 12px;
      color: var(--fg-muted);
    }
    .icon-button {
      width: 32px;
      height: 32px;
      border-radius: 999px;
      border: 1px solid var(--border);
      background: transparent;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      color: var(--fg);
      cursor: pointer;
      transition: background .18s ease, color .18s ease, border-color .18s ease, transform .12s ease;
    }
    .icon-button:hover {
      background: var(--fg);
      color: #000;
      border-color: var(--fg);
      transform: translateY(-1px);
    }
    .icon {
      width: 16px;
      height: 16px;
      display: inline-block;
    }
    .section-label {
      font-size: 11px;
      letter-spacing: .16em;
      text-transform: uppercase;
      color: var(--fg-muted);
      margin-bottom: 8px;
    }
    .card {
      border-radius: 12px;
      border: 1px solid var(--border);
      padding: 14px 14px 12px;
      background: linear-gradient(135deg, #050505, #000);
    }
    .field {
      display: flex;
      flex-direction: column;
      gap: 6px;
      margin-bottom: 10px;
    }
    label {
      font-size: 12px;
      color: var(--fg-muted);
    }
    input[type="text"],
    textarea {
      width: 100%;
      border-radius: 10px;
      border: 1px solid var(--border);
      background: rgba(0, 0, 0, 0.9);
      color: var(--fg);
      padding: 9px 10px;
      font-size: 14px;
      outline: none;
      transition: border-color .16s ease, box-shadow .16s ease, background .16s ease;
      resize: vertical;
      min-height: 38px;
    }
    textarea {
      min-height: 88px;
    }
    input::placeholder,
    textarea::placeholder {
      color: var(--fg-muted);
    }
    input:focus,
    textarea:focus {
      border-color: var(--accent);
      box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.3);
      background: #050505;
    }
    .row {
      display: flex;
      justify-content: space-between;
      align-items: center;
      gap: 12px;
      margin-top: 6px;
    }
    .hint {
      font-size: 11px;
      color: var(--fg-muted);
    }
    .primary-btn {
      border-radius: 999px;
      border: 1px solid var(--fg);
      background: var(--fg);
      color: #000;
      padding: 8px 18px;
      font-size: 14px;
      font-weight: 500;
      cursor: pointer;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: 6px;
      min-width: 108px;
      transition: background .16s ease, color .16s ease, transform .1s ease, box-shadow .16s ease, border-color .16s ease;
    }
    .primary-btn span {
      font-size: 12px;
      letter-spacing: .12em;
      text-transform: uppercase;
    }
    .primary-btn:hover {
      background: #e6e6e6;
      border-color: #e6e6e6;
      box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.2);
      transform: translateY(-1px);
    }
    .primary-btn:active {
      transform: translateY(0);
      box-shadow: none;
    }
    .primary-btn[disabled] {
      cursor: default;
      background: var(--accent-muted);
      border-color: var(--accent-muted);
      color: #111;
      box-shadow: none;
      transform: none;
    }
    .status-line {
      margin-top: 10px;
      min-height: 18px;
      font-size: 12px;
      color: var(--fg-muted);
      display: flex;
      align-items: center;
      gap: 8px;
    }
    .dot {
      width: 6px;
      height: 6px;
      border-radius: 999px;
      border: 1px solid var(--fg-muted);
      background: transparent;
    }
    .dot.success {
      border-color: var(--success);
      background: var(--success);
    }
    .dot.error {
      border-color: var(--error);
      background: var(--error);
    }
    .footer {
      margin-top: 18px;
      display: flex;
      justify-content: space-between;
      align-items: center;
      font-size: 11px;
      color: var(--fg-muted);
    }
    .link {
      color: var(--fg);
      text-decoration: none;
      border-bottom: 1px solid transparent;
      padding-bottom: 1px;
      cursor: pointer;
      transition: border-color .12s ease, color .12s ease, opacity .12s ease;
    }
    .link:hover {
      border-color: var(--fg);
      opacity: .9;
    }
    @media (max-width: 640px) {
      body {
        padding: 12px;
      }
      .app {
        padding: 18px 16px 16px;
        border-radius: 14px;
      }
    }
  </style>
</head>
<body>
  <div class="app">
    <div class="header">
      <div class="title-group">
        <div class="title">SMS Forwarder</div>
        <div class="subtitle">通过 Modem 发送短信</div>
      </div>
      <button class="icon-button" id="settingsBtn" title="配置">
        <span class="icon" aria-hidden="true">
          <!-- 简单设置图标（齿轮） -->
          <svg viewBox="0 0 24 24" width="16" height="16" fill="none" xmlns="http://www.w3.org/2000/svg">
            <circle cx="12" cy="12" r="3.2" stroke="currentColor" stroke-width="1.4"/>
            <path d="M4.9 13.1c-.06-.36-.09-.73-.09-1.1 0-.37.03-.74.09-1.1l-1.67-1.3a.4.4 0 0 1-.1-.5l1.58-2.74c.1-.17.32-.24.5-.17l1.96.79a6.5 6.5 0 0 1 1.9-1.1l.3-2.08A.4.4 0 0 1 9.9 3h4.2c.2 0 .37.15.4.34l.3 2.08c.7.25 1.34.62 1.9 1.1l1.96-.8a.4.4 0 0 1 .5.18l1.58 2.74a.4.4 0 0 1-.1.5l-1.67 1.3c.06.36.09.73.09 1.1 0 .37-.03.74-.09 1.1l1.67 1.3c.16.12.2.35.1.52l-1.58 2.73a.4.4 0 0 1-.5.18l-1.96-.8a6.5 6.5 0 0 1-1.9 1.1l-.3 2.08a.4.4 0 0 1-.4.34H9.9a.4.4 0 0 1-.4-.34l-.3-2.08a6.5 6.5 0 0 1-1.9-1.1l-1.96.8a.4.4 0 0 1-.5-.18L3.26 15a.4.4 0 0 1 .1-.5L4.9 13.1Z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"/>
          </svg>
        </span>
      </button>
    </div>

    <div class="section-label">Compose</div>
    <div class="card">
      <div class="field">
        <label for="number">收信号码</label>
        <input id="number" type="text" placeholder="+8613800138000 或本地号码">
      </div>
      <div class="field">
        <label for="text">短信内容</label>
        <textarea id="text" placeholder="输入要发送的短信内容"></textarea>
      </div>
      <div class="row">
        <div class="hint" id="hint">短信会通过 Modem 直接发送。</div>
        <button class="primary-btn" id="sendBtn">
          <span>Send</span>
        </button>
      </div>
      <div class="status-line" id="statusLine">
        <span class="dot" id="statusDot"></span>
        <span id="statusText">等待发送</span>
      </div>
    </div>

    <div class="footer">
      <span>HTTP API: <code>/send</code> / <code>/config</code></span>
      <a class="link" href="/settings">配置</a>
    </div>
  </div>

  <script>
    (function () {
      const sendBtn = document.getElementById('sendBtn');
      const numberInput = document.getElementById('number');
      const textInput = document.getElementById('text');
      const statusText = document.getElementById('statusText');
      const statusDot = document.getElementById('statusDot');
      const settingsBtn = document.getElementById('settingsBtn');

      function setStatus(kind, msg) {
        statusDot.className = 'dot' + (kind ? ' ' + kind : '');
        statusText.textContent = msg;
      }

      async function sendSms() {
        const number = numberInput.value.trim();
        const text = textInput.value.trim();
        if (!number) {
          setStatus('error', '请输入收信号码');
          numberInput.focus();
          return;
        }
        if (!text) {
          setStatus('error', '请输入短信内容');
          textInput.focus();
          return;
        }
        sendBtn.disabled = true;
        setStatus('', '发送中...');
        try {
          const res = await fetch('/send', {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json'
            },
            body: JSON.stringify([number, text])
          });
          if (!res.ok) {
            throw new Error('HTTP ' + res.status);
          }
          setStatus('success', '发送指令已提交（请查看日志确认结果）');
        } catch (e) {
          console.error(e);
          setStatus('error', '发送失败：' + (e.message || e));
        } finally {
          sendBtn.disabled = false;
        }
      }

      sendBtn.addEventListener('click', function () {
        sendSms();
      });
      textInput.addEventListener('keydown', function (ev) {
        if ((ev.metaKey || ev.ctrlKey) && ev.key === 'Enter') {
          ev.preventDefault();
          sendSms();
        }
      });
      settingsBtn.addEventListener('click', function () {
        window.location.href = '/settings';
      });
    })();
  </script>
</body>
</html>
"#;

const SETTINGS_HTML: &str = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <title>SMS Forwarder - 配置</title>
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <style>
    :root {
      color-scheme: light dark;
      --bg: #000;
      --fg: #fff;
      --fg-muted: #aaa;
      --border: #333;
      --accent: #fff;
      --accent-muted: #666;
      --error: #ff4d4f;
      --success: #52c41a;
    }
    * {
      box-sizing: border-box;
    }
    body {
      margin: 0;
      font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      background-color: var(--bg);
      color: var(--fg);
      min-height: 100vh;
      display: flex;
      align-items: center;
      justify-content: center;
    }
    .app {
      width: 100%;
      max-width: 640px;
      padding: 24px 20px 20px;
      border: 1px solid var(--border);
      border-radius: 16px;
      background: radial-gradient(circle at top left, #111 0, #000 40%);
      box-shadow: 0 18px 45px rgba(0, 0, 0, 0.7);
    }
    .header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      margin-bottom: 18px;
    }
    .title-group {
      display: flex;
      flex-direction: column;
      gap: 4px;
    }
    .title {
      font-size: 20px;
      font-weight: 600;
      letter-spacing: .04em;
      text-transform: uppercase;
    }
    .subtitle {
      font-size: 12px;
      color: var(--fg-muted);
    }
    .icon-button {
      width: 32px;
      height: 32px;
      border-radius: 999px;
      border: 1px solid var(--border);
      background: transparent;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      color: var(--fg);
      cursor: pointer;
      transition: background .18s ease, color .18s ease, border-color .18s ease, transform .12s ease;
    }
    .icon-button:hover {
      background: var(--fg);
      color: #000;
      border-color: var(--fg);
      transform: translateY(-1px);
    }
    .section-label {
      font-size: 11px;
      letter-spacing: .16em;
      text-transform: uppercase;
      color: var(--fg-muted);
      margin-bottom: 8px;
    }
    .grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(230px, 1fr));
      gap: 10px 18px;
    }
    .field {
      display: flex;
      flex-direction: column;
      gap: 6px;
      margin-bottom: 8px;
    }
    label {
      font-size: 12px;
      color: var(--fg-muted);
    }
    input[type="text"],
    input[type="number"],
    textarea {
      width: 100%;
      border-radius: 10px;
      border: 1px solid var(--border);
      background: rgba(0, 0, 0, 0.9);
      color: var(--fg);
      padding: 8px 10px;
      font-size: 13px;
      outline: none;
      transition: border-color .16s ease, box-shadow .16s ease, background .16s ease;
      resize: vertical;
      min-height: 36px;
    }
    textarea {
      min-height: 80px;
    }
    input::placeholder,
    textarea::placeholder {
      color: var(--fg-muted);
    }
    input:focus,
    textarea:focus {
      border-color: var(--accent);
      box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.3);
      background: #050505;
    }
    .note {
      font-size: 11px;
      color: var(--fg-muted);
    }
    .row {
      display: flex;
      justify-content: space-between;
      align-items: center;
      gap: 12px;
      margin-top: 10px;
    }
    .primary-btn {
      border-radius: 999px;
      border: 1px solid var(--fg);
      background: var(--fg);
      color: #000;
      padding: 8px 18px;
      font-size: 14px;
      font-weight: 500;
      cursor: pointer;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: 6px;
      min-width: 108px;
      transition: background .16s ease, color .16s ease, transform .1s ease, box-shadow .16s ease, border-color .16s ease;
    }
    .primary-btn span {
      font-size: 12px;
      letter-spacing: .12em;
      text-transform: uppercase;
    }
    .primary-btn:hover {
      background: #e6e6e6;
      border-color: #e6e6e6;
      box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.2);
      transform: translateY(-1px);
    }
    .primary-btn:active {
      transform: translateY(0);
      box-shadow: none;
    }
    .primary-btn[disabled] {
      cursor: default;
      background: var(--accent-muted);
      border-color: var(--accent-muted);
      color: #111;
      box-shadow: none;
      transform: none;
    }
    .status-line {
      margin-top: 10px;
      min-height: 18px;
      font-size: 12px;
      color: var(--fg-muted);
      display: flex;
      align-items: center;
      gap: 8px;
    }
    .dot {
      width: 6px;
      height: 6px;
      border-radius: 999px;
      border: 1px solid var(--fg-muted);
      background: transparent;
    }
    .dot.success {
      border-color: var(--success);
      background: var(--success);
    }
    .dot.error {
      border-color: var(--error);
      background: var(--error);
    }
    .footer {
      margin-top: 18px;
      display: flex;
      justify-content: space-between;
      align-items: center;
      font-size: 11px;
      color: var(--fg-muted);
      flex-wrap: wrap;
      gap: 8px;
    }
    .link {
      color: var(--fg);
      text-decoration: none;
      border-bottom: 1px solid transparent;
      padding-bottom: 1px;
      cursor: pointer;
      transition: border-color .12s ease, color .12s ease, opacity .12s ease;
    }
    .link:hover {
      border-color: var(--fg);
      opacity: .9;
    }
    @media (max-width: 640px) {
      body {
        padding: 12px;
      }
      .app {
        padding: 18px 16px 16px;
        border-radius: 14px;
      }
    }
  </style>
</head>
<body>
  <div class="app">
    <div class="header">
      <div class="title-group">
        <div class="title">配置</div>
        <div class="subtitle">管理 Bark / Modem 参数</div>
      </div>
      <button class="icon-button" id="backBtn" title="返回发送短信">
        <!-- 返回图标 -->
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M10.5 6L5 11.5L10.5 17" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/>
          <path d="M6 11.5H19" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </button>
    </div>

    <div class="section-label">Bark &amp; Security</div>
    <div class="grid">
      <div class="field">
        <label for="bark_key">Bark Key</label>
        <input id="bark_key" type="text" autocomplete="off" placeholder="Bark 推送 key">
      </div>
      <div class="field">
        <label for="aes_key">AES Key (16 字节)</label>
        <input id="aes_key" type="text" autocomplete="off" placeholder="16 字节字符串">
      </div>
      <div class="field">
        <label for="aes_iv">AES IV (16 字节)</label>
        <input id="aes_iv" type="text" autocomplete="off" placeholder="16 字节字符串">
      </div>
    </div>
    <div class="field">
      <label for="emergency_keywords">紧急关键词（每行一个）</label>
      <textarea id="emergency_keywords" placeholder="例如：&#10;违规停车&#10;验证码"></textarea>
      <div class="note">短信内容包含任一关键字时，将以紧急通知推送到 Bark。</div>
    </div>

    <div class="section-label" style="margin-top: 10px;">Modem</div>
    <div class="grid">
      <div class="field">
        <label for="modem_device">串口设备路径</label>
        <input id="modem_device" type="text" autocomplete="off" placeholder="/dev/ttyUSB2">
      </div>
      <div class="field">
        <label for="baud_rate">波特率</label>
        <input id="baud_rate" type="number" min="1" step="1" placeholder="115200">
      </div>
    </div>

    <div class="row">
      <div class="note">保存后会写入 <code>config.json</code>，下次启动生效；部分参数也会实时使用。</div>
      <button class="primary-btn" id="saveBtn"><span>Save</span></button>
    </div>
    <div class="status-line" id="statusLine">
      <span class="dot" id="statusDot"></span>
      <span id="statusText">正在加载当前配置...</span>
    </div>

    <div class="footer">
      <span>接口：<code>GET /config</code> / <code>POST /config</code></span>
      <a class="link" href="/">返回发送短信</a>
    </div>
  </div>

  <script>
    (function () {
      const ids = [
        'bark_key',
        'aes_key',
        'aes_iv',
        'emergency_keywords',
        'modem_device',
        'baud_rate'
      ];

      const el = {};
      ids.forEach(function (id) {
        el[id] = document.getElementById(id);
      });

      const statusText = document.getElementById('statusText');
      const statusDot = document.getElementById('statusDot');
      const saveBtn = document.getElementById('saveBtn');
      const backBtn = document.getElementById('backBtn');

      function setStatus(kind, msg) {
        statusDot.className = 'dot' + (kind ? ' ' + kind : '');
        statusText.textContent = msg;
      }

      function configFromForm() {
        const keywordsRaw = el.emergency_keywords.value
          .split(/[\r\n]+/)
          .map(function (s) { return s.trim(); })
          .filter(function (s) { return s.length > 0; });
        const baud = parseInt(el.baud_rate.value, 10);
        return {
          bark_key: el.bark_key.value.trim(),
          aes_key: el.aes_key.value.trim(),
          aes_iv: el.aes_iv.value.trim(),
          emergency_keywords: keywordsRaw,
          modem_device: el.modem_device.value.trim() || "/dev/ttyUSB2",
          baud_rate: Number.isFinite(baud) && baud > 0 ? baud : 115200
        };
      }

      function fillForm(cfg) {
        el.bark_key.value = cfg.bark_key || '';
        el.aes_key.value = cfg.aes_key || '';
        el.aes_iv.value = cfg.aes_iv || '';
        el.emergency_keywords.value = (cfg.emergency_keywords || []).join('\n');
        el.modem_device.value = cfg.modem_device || '/dev/ttyUSB2';
        el.baud_rate.value = cfg.baud_rate != null ? String(cfg.baud_rate) : '115200';
      }

      async function loadConfig() {
        setStatus('', '正在加载当前配置...');
        try {
          const res = await fetch('/config');
          if (!res.ok) {
            throw new Error('HTTP ' + res.status);
          }
          const cfg = await res.json();
          fillForm(cfg);
          setStatus('success', '配置已加载');
        } catch (e) {
          console.error(e);
          setStatus('error', '加载失败：' + (e.message || e));
        }
      }

      async function saveConfig() {
        const cfg = configFromForm();
        saveBtn.disabled = true;
        setStatus('', '保存中...');
        try {
          const res = await fetch('/config', {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json'
            },
            body: JSON.stringify(cfg)
          });
          if (!res.ok) {
            throw new Error('HTTP ' + res.status);
          }
          setStatus('success', '保存成功（已写入 config.json）');
        } catch (e) {
          console.error(e);
          setStatus('error', '保存失败：' + (e.message || e));
        } finally {
          saveBtn.disabled = false;
        }
      }

      saveBtn.addEventListener('click', function () {
        saveConfig();
      });
      backBtn.addEventListener('click', function () {
        window.location.href = '/';
      });

      loadConfig();
    })();
  </script>
</body>
</html>
"#;
