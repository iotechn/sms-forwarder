# SMS Forwarder

将 Modem 短信通过 **AT 指令**（串口直连）轮询并转发到 [Bark](https://github.com/Finb/Bark)（iOS 推送），支持按内容关键字触发紧急通知；同时提供 HTTP API 用于查看/修改配置和发送短信。

## 前置条件

- **Rust**（建议 1.70+）：用于编译运行
- **串口设备**：USB 4G 网卡或 GSM 模块对应的串口（如 `/dev/ttyUSB2`），需有读写权限；**不需要** ModemManager/mmcli
- **Bark**：在 iOS 设备上安装 Bark，并在 Bark 服务端或自建服务器获取 `key`，用于推送

## 配置

在项目根目录（或运行时的当前工作目录）放置 `config.json`，格式如下：

```json
{
  "bark_key": "你的Bark密钥",
  "aes_key": "16字节AES密钥字符串",
  "aes_iv": "16字节AES-IV字符串",
  "emergency_keywords": ["违规停车", "验证码"],
  "modem_device": "/dev/ttyUSB2",
  "baud_rate": 115200
}
```

- **bark_key**：Bark 的 key，推送时会用到
- **aes_key** / **aes_iv**：Bark 服务若开启加密推送，需与服务器端一致的 16 字节 key/iv（字符串长度 16）
- **emergency_keywords**：关键字列表；**短信内容**包含其中任意一个时，会以紧急级别推送到 Bark（如 critical、高音量等）
- **modem_device**（可选）：Modem 串口设备路径，默认 `/dev/ttyUSB2`
- **baud_rate**（可选）：串口波特率，默认 `115200`

## 构建与运行

```bash
cargo build --release
./target/release/sms-forwarder
```

### 静态编译（推荐用于部署）

在 GLIBC 较旧或不同发行版上运行时，建议用 **musl** 目标做静态链接，生成不依赖系统 GLIBC 的单一可执行文件。

**方式一：用 cross 构建（推荐，无需本机安装 musl）**

需安装 [cross](https://github.com/cross-rs/cross) 与 Docker，在容器内完成编译，不依赖本机 musl-gcc：

```bash
cargo install cross
rustup target add x86_64-unknown-linux-musl
cross build --release --target x86_64-unknown-linux-musl
```

**方式二：本机安装 musl 后构建**

```bash
rustup target add x86_64-unknown-linux-musl
# 安装 musl-gcc（用于编译 ring/rustls 的 C 代码）
# Debian/Ubuntu:
sudo apt install musl-tools
# Fedora: sudo dnf install musl-gcc
# Alpine: apk add musl-dev

# 然后二选一：
CC_x86_64_unknown_linux_musl=musl-gcc cargo build --release --target x86_64-unknown-linux-musl
# 或
./build-static.sh
```

生成的二进制在 `target/x86_64-unknown-linux-musl/release/sms-forwarder`，可直接拷贝到任意 x86_64 Linux 运行，无需同版本 GLIBC。

或开发时直接运行：

```bash
cargo run
```

**日志**：默认输出 `info` 级别。可通过环境变量调节，例如：
- `RUST_LOG=debug ./target/release/sms-forwarder` — 更详细（含 API、Bark 请求等）
- `RUST_LOG=sms_forwarder=info` — 仅本 crate 的 info

程序会：

1. 在后台每 5 秒通过 **AT 指令**（串口）轮询 modem 上的短信（`AT+CMGL="ALL"`），解析后转发到 Bark，并删除已处理短信（`AT+CMGD`）
2. 在 `0.0.0.0:10086` 启动 HTTP 服务，提供下面 API

**注意**：工作目录需能读写当前目录下的 `config.json`（启动时读取，修改配置时写入）。

## HTTP API

- **GET /config**  
  返回当前内存中的配置（JSON，即 `Config` 结构体）。

- **POST /config**  
  用请求体中的 JSON 覆盖并保存配置到 `config.json`。请求体格式与上面 `config.json` 相同。

- **POST /send**  
  通过 modem 发送一条短信。请求体为 JSON 数组：`[ "号码", "短信内容" ]`。  
  例如：  
  `["+8613800138000", "Hello"]`

## 行为说明

- 短信轮询与发送均通过**串口 AT 指令**直连 modem（`config.json` 中 `modem_device`，默认 `/dev/ttyUSB2`）；使用文本模式 `AT+CMGF=1`，列表 `AT+CMGL="ALL"`，发送 `AT+CMGS`，删除 `AT+CMGD`
- 转发到 Bark 时，标题为 `SMS from <号码>`，正文为短信内容；若**正文包含** `emergency_keywords` 中任一关键字，会带上紧急级别参数
- 已成功转发的短信会从 modem 中删除（`AT+CMGD=<index>`），避免重复推送

## License

按项目仓库约定（如有）；无则视为私有/未指定。
