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

### 服务端

启动一个发现服务，响应客户端的发现请求：

#### 基础用法

```rust
use simple_discover::DiscoverServer;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    // 使用默认配置创建服务器
    let server = DiscoverServer::new();
    
    // 启动服务
    let handle = server.start().await?;
    
    println!("Server is running. Press Ctrl+C to stop.");
    
    // 等待 Ctrl+C 信号
    signal::ctrl_c().await?;
    
    // 优雅关闭
    server.stop();
    let _ = handle.await;
    
    Ok(())
}
```

#### 带自定义数据的服务端

服务器可以在响应中携带自定义元数据，例如服务名称、版本、能力等信息：

```rust
use simple_discover::DiscoverServer;
use serde_json::json;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    // 定义自定义数据（必须是有效的 JSON）
    let custom_data = json!({
        "name": "My Service",
        "version": "1.0.0",
        "capabilities": ["http", "grpc", "websocket"],
        "metadata": {
            "region": "cn-north-1",
            "environment": "production",
            "tags": ["api", "backend"]
        }
    });
    
    // 创建带自定义数据的服务器
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

```rust
use std::net::Ipv4Addr;
use simple_discover::{DiscoverConfig, DiscoverServer};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    // 创建自定义配置
    let config = DiscoverConfig::new()
        .set_multicast_addr(Ipv4Addr::new(224, 0, 0, 200))
        .set_ports(vec![20001, 20010, 20100])
        .set_listen_addr(Ipv4Addr::new(192, 168, 1, 100)); // 设置自定义监听地址
    
    // 在服务器上设置自定义数据
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

### 客户端

发起设备发现请求，获取局域网内所有响应的设备及其自定义数据：

#### 基础用法

```rust
use simple_discover::DiscoverClient;
use std::time::Duration;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    // 使用默认配置创建客户端
    let client = DiscoverClient::new();
    
    // 发起发现请求，超时时间5秒
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

#### 自定义配置

```rust
use std::net::Ipv4Addr;
use std::time::Duration;
use simple_discover::{DiscoverConfig, DiscoverClient};

#[tokio::main]
async fn main() {
    env_logger::init();
    
    // 创建自定义配置
    let config = DiscoverConfig::new()
        .set_multicast_addr(Ipv4Addr::new(224, 0, 0, 200))
        .set_ports(vec![20001, 20010, 20100]);
    
    let client = DiscoverClient::with_config(config);
    
    // 发起发现请求
    let devices = client.discover(Duration::from_secs(5)).await.unwrap();
    
    for device in devices {
        println!("发现设备: {} {:?}", device.ip, device.data);
    }
}
```

## 📖 API 说明

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
    .set_listen_addr(Ipv4Addr::new(192, 168, 1, 100)); // 指定监听特定 IP
```

### 服务器自定义数据

`DiscoverServer` 支持通过 `with_custom_data()` 方法设置响应时携带的自定义数据：

```rust
let server = DiscoverServer::new()
    .with_custom_data(serde_json::json!({"key": "value"}));
```

或者与配置结合使用：

```rust
let config = DiscoverConfig::new()
    .set_multicast_addr(Ipv4Addr::new(224, 0, 0, 200))
    .set_ports(vec![20001, 20010, 20100]);

let server = DiscoverServer::with_config(config)
    .with_custom_data(serde_json::json!({"service": "my-app"}));
```

### 返回类型

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

**注意**：如果 `custom_data` 为 `None`，服务器响应中将不包含 `data` 字段，以减少网络传输开销。

## 🔧 协议设计

为了解决 UDP 的粘包和丢包问题，simple-discover 使用了自定义的应用层协议：

```
+--------+---------+------+--------+--------+----------+
| Magic  | Version | Type | Length | CRC32  | Payload  |
| 4 bytes| 1 byte  |1 byte|4 bytes |4 bytes | variable |
+--------+---------+------+--------+--------+----------+
```

- **Magic**: `0x53445343` ("SDSC:Simple Discover Service Code") - 协议魔数，用于识别有效数据包
- **Version**: 协议版本号（当前为 1）
- **Type**: 消息类型（0=Request, 1=Response）
- **Length**: Payload 长度（字节）
- **CRC32**: Payload 的 CRC32 校验码，确保数据完整性
- **Payload**: JSON 格式的实际数据

### 消息格式

**请求消息：**
```json
{"type":"request"}
```

**响应消息：**
```json
{
  "type": "response",
  "data": {...}  // 可选，仅在服务器配置了 custom_data 时存在
}
```

## 🧪 运行示例

在项目根目录下打开两个终端：

```bash
# 终端1：启动服务端
cargo run --example server

# 终端2：运行客户端（在另一个终端中）
cargo run --example client
```