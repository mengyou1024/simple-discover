# Simple Discover

一个基于 Rust 的轻量级服务发现库，使用 UDP 组播实现局域网内的设备自动发现功能。支持自定义数据传递和可靠的应用层协议。

## ✨ 特性

- 🔍 **自动发现**：零配置，自动发现局域网内的设备
- 📦 **自定义数据**：服务器可在响应中携带任意 JSON 格式的自定义数据
- 🔄 **多端口容错**：支持多个端口（10001, 10010, 10100, 11000），防止端口被占用
- ✅ **数据校验**：使用 CRC32 校验确保数据完整性
- 📋 **可靠协议**：自定义应用层协议，解决 UDP 粘包和丢包问题
- ⚙️ **高度可配置**：支持自定义组播地址、端口列表和元数据

## 📦 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
simple-discover = "*"
serde_json = "*"
tokio = { version = "*", features = ["full"] }
```

## 🚀 快速开始

### 基础示例

#### 服务端

最简单的服务端实现：

```rust
use simple_discover::DiscoverServer;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let server = DiscoverServer::new();
    let handle = server.start().await?;
    
    println!("Server is running. Press Ctrl+C to stop.");
    signal::ctrl_c().await?;
    
    server.stop();
    let _ = handle.await;
    Ok(())
}
```

#### 客户端

最简单的客户端实现：

```rust
use simple_discover::DiscoverClient;
use std::time::Duration;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    let client = DiscoverClient::new();
    let devices = client.discover(Duration::from_secs(5)).await.unwrap();
    
    println!("发现了 {} 个设备:\n", devices.len());
    for (i, device) in devices.iter().enumerate() {
        println!("设备 {}: IP = {}", i + 1, device.ip);
    }
}
```

### 进阶用法

#### 带自定义数据的服务端

服务器可以在响应中携带自定义元数据（如服务名称、版本、能力等）：

```rust
use simple_discover::DiscoverServer;
use serde_json::json;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let custom_data = json!({
        "name": "My Service",
        "version": "1.0.0",
        "capabilities": ["http", "grpc", "websocket"],
        "metadata": {
            "region": "cn-north-1",
            "environment": "production"
        }
    });
    
    let server = DiscoverServer::new()
        .with_custom_data(custom_data);
    
    let handle = server.start().await?;
    println!("Server is running with custom data.");
    
    signal::ctrl_c().await?;
    server.stop();
    let _ = handle.await;
    Ok(())
}
```

#### 完整自定义配置

同时自定义网络配置和服务数据：

```rust
use std::net::Ipv4Addr;
use simple_discover::{DiscoverConfig, DiscoverServer};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    // 自定义网络配置
    let config = DiscoverConfig::new()
        .set_multicast_addr(Ipv4Addr::new(224, 0, 0, 200))
        .set_ports(vec![20001, 20010, 20100])
        .set_listen_addr(Ipv4Addr::new(192, 168, 1, 100));
    
    // 创建带自定义数据的服务器
    let server = DiscoverServer::with_config(config)
        .with_custom_data(json!({
            "service": "my-app",
            "port": 8080
        }));
    
    let handle = server.start().await?;
    handle.await?;
    Ok(())
}
```

#### 客户端接收自定义数据

客户端可以获取服务器返回的自定义数据：

```rust
use simple_discover::DiscoverClient;
use std::time::Duration;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    let client = DiscoverClient::new();
    let devices = client.discover(Duration::from_secs(5)).await.unwrap();
    
    println!("发现了 {} 个设备:\n", devices.len());
    
    for (i, device) in devices.iter().enumerate() {
        println!("设备 {}:", i + 1);
        println!("  IP: {}", device.ip);
        
        if let Some(data) = &device.data {
            println!("  数据: {}", serde_json::to_string_pretty(data).unwrap());
        } else {
            println!("  数据: <无>");
        }
        println!();
    }
}
```

输出示例：
```
发现了 2 个设备:

设备 1:
  IP: 192.168.1.100
  数据: {
    "name": "My Service",
    "version": "1.0.0",
    "capabilities": ["http", "grpc"]
  }

设备 2:
  IP: 192.168.1.101
  数据: <无>
```

## 📖 API 参考

### 配置项

`DiscoverConfig` 结构体包含以下可配置项：

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `multicast_addr` | `Ipv4Addr` | `224.0.0.100` | 组播地址 |
| `ports` | `Vec<u16>` | `[10001, 10010, 10100, 11000]` | 监听的端口列表 |
| `listen_addr` | `Ipv4Addr` | `0.0.0.0` (UNSPECIFIED) | 自定义监听地址（设置为 UNSPECIFIED 时自动选择） |

使用 builder 模式配置：

```rust
let config = DiscoverConfig::new()
    .set_multicast_addr(Ipv4Addr::new(224, 0, 0, 200))
    .set_ports(vec![20001, 20010, 20100, 21000])
    .set_listen_addr(Ipv4Addr::new(192, 168, 1, 100));
```

### 服务器 API

#### 创建服务器

```rust
// 使用默认配置
let server = DiscoverServer::new();

// 使用自定义配置
let config = DiscoverConfig::new()
    .set_multicast_addr(Ipv4Addr::new(224, 0, 0, 200));
let server = DiscoverServer::with_config(config);
```

#### 设置自定义数据

```rust
let server = DiscoverServer::new()
    .with_custom_data(serde_json::json!({"key": "value"}));
```

或者与配置结合使用：

```rust
let server = DiscoverServer::with_config(config)
    .with_custom_data(serde_json::json!({"service": "my-app"}));
```

#### 启动和停止

```rust
// 启动服务器
let handle = server.start().await?;

// 停止服务器
server.stop();
let _ = handle.await;
```

### 客户端 API

#### 创建客户端

```rust
// 使用默认配置
let client = DiscoverClient::new();

// 使用自定义配置
let config = DiscoverConfig::new()
    .set_multicast_addr(Ipv4Addr::new(224, 0, 0, 200));
let client = DiscoverClient::with_config(config);
```

#### 发起发现请求

```rust
let devices = client.discover(Duration::from_secs(5)).await?;
```

### 数据结构

#### `DiscoveredDevice`

客户端发现设备后返回的结构体：

```rust
pub struct DiscoveredDevice {
    /// 设备 IP 地址
    pub ip: Ipv4Addr,
    /// 自定义数据（如果服务器提供了的话）
    pub data: Option<serde_json::Value>,
}
```

### 自定义数据格式

自定义数据可以是任何可以被序列化为 JSON 的类型：

```rust
use serde_json::json;

// 简单对象
let data1 = json!({"name": "service-a"});

// 嵌套对象
let data2 = json!({
    "service": "api-gateway",
    "config": {
        "port": 8080,
        "protocol": "https"
    },
    "tags": ["production", "v1"]
});

// 数组
let data3 = json!(["feature1", "feature2", "feature3"]);

// 基本类型
let data4 = json!("just a string");
let data5 = json!(42);
```

**注意**：
- 如果 `custom_data` 为 `None`，服务器响应中的 Payload 将为空
- Payload 直接是 `custom_data` 的 JSON 序列化结果

## 🔧 协议设计

为了解决 UDP 的粘包和丢包问题，simple-discover 使用了自定义的应用层协议：

### 协议头部结构

```
+--------+------+--------+--------+----------+
| Magic  | Type | Length | CRC32  | Payload  |
| 4 bytes|1 byte|4 bytes |4 bytes | variable |
+--------+------+--------+--------+----------+
```

### 字段说明

- **Magic**: `0x53445343` ("SDSC:Simple Discover Service Code") - 协议魔数，用于识别有效数据包
- **Type**: 消息类型（0=Request, 1=Response）
- **Length**: Payload 长度（字节）
- **CRC32**: Payload 的 CRC32 校验码，确保数据完整性
- **Payload**: JSON 格式的实际数据（可选）

### 消息格式

#### 请求消息

- **用途**：客户端发起设备发现请求
- **Payload**：为空（长度为 0）

```
Header: Magic + Type(0) + Length(0) + CRC32(empty)
Payload: (empty)
```

#### 响应消息

- **用途**：服务器响应发现请求
- **Payload**：
  - 如果配置了 `custom_data`：直接是 `custom_data` 的 JSON 序列化结果
  - 如果未配置 `custom_data`：为空（长度为 0）

带自定义数据的响应：
```
Header: Magic + Type(1) + Length(N) + CRC32(data)
Payload: {"name": "My Service", "version": "1.0.0"}
```

无自定义数据的响应：
```
Header: Magic + Type(1) + Length(0) + CRC32(empty)
Payload: (empty)
```

### 设计原则

1. **简洁性**：移除了冗余的 Version 和 type 字段，减少协议开销
2. **职责分离**：消息类型由 Header 中的 Type 字段管理，Payload 只承载业务数据
3. **灵活性**：允许服务器直接发送任意 JSON 结构，无需额外包装
4. **可靠性**：通过 CRC32 校验确保数据完整性

## 🧪 运行示例

在项目根目录下打开两个终端：

```bash
# 终端1：启动服务端
cargo run --example server

# 终端2：运行客户端（在另一个终端中）
cargo run --example client
```

## 📝 应用场景

- **微服务发现**：局域网内微服务的自动发现和注册
- **P2P 网络**：点对点应用中的节点发现
- **IoT 设备管理**：物联网设备的自动发现和配置
- **分布式系统**：集群节点的动态加入和离开
- **开发工具**：本地开发环境中的服务自动连接

## ⚠️ 注意事项

1. **局域网限制**：仅适用于局域网（LAN）环境，依赖 UDP 组播支持
2. **防火墙配置**：确保防火墙允许组播流量和配置的端口
3. **跨子网限制**：组播通常不能跨越不同子网
4. **数据大小**：自定义数据不宜过大，建议控制在几 KB 以内
5. **安全性**：未提供加密或认证机制，仅适用于可信网络环境
6. **序列化要求**：`with_custom_data()` 要求数据必须能成功序列化为 JSON，避免使用特殊浮点数（如 `NaN`、`Infinity`）

## 📄 许可证

本项目采用 MIT 许可证。

## 🤖 AI 辅助开发

本项目在 AI 辅助下开发，使用阿里云通义灵码进行代码生成、重构和文档编写。

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！