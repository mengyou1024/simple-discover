# Simple Discover

A lightweight service discovery library built with Rust, using UDP multicast for automatic device discovery within local area networks. Supports custom data transmission and reliable application-layer protocol.

## ✨ Features

- 🔍 **Auto Discovery**: Zero configuration, automatically discovers devices in the LAN
- 📦 **Custom Data**: Servers can carry arbitrary JSON-formatted custom data in responses
- 🔄 **Multi-Port Fault Tolerance**: Supports multiple ports (10001, 10010, 10100, 11000) to prevent port conflicts
- ✅ **Data Integrity**: Uses CRC32 checksums to ensure data integrity
- 📋 **Reliable Protocol**: Custom application-layer protocol to solve UDP packet sticking and loss issues
- ⚙️ **Highly Configurable**: Supports custom multicast addresses, port lists, and metadata

## 📦 Installation

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
simple-discover = "*"
serde_json = "*"
tokio = { version = "*", features = ["full"] }
```

## 🚀 Quick Start

### Server

Start a discovery service that responds to client discovery requests:

#### Basic Usage

```rust
use simple_discover::DiscoverServer;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    // Create server with default configuration
    let server = DiscoverServer::new();
    
    // Start the service
    let handle = server.start().await?;
    
    println!("Server is running. Press Ctrl+C to stop.");
    
    // Wait for Ctrl+C signal
    signal::ctrl_c().await?;
    
    // Graceful shutdown
    server.stop();
    let _ = handle.await;
    
    Ok(())
}
```

#### Server with Custom Data

Servers can carry custom metadata in responses, such as service name, version, capabilities, etc.:

```rust
use simple_discover::DiscoverServer;
use serde_json::json;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    // Define custom data (must be valid JSON)
    let custom_data = json!({
        "name": "My Service",
        "version": "1.0.0",
        "capabilities": ["http", "grpc", "websocket"],
        "metadata": {
            "region": "us-east-1",
            "environment": "production",
            "tags": ["api", "backend"]
        }
    });
    
    // Create server with custom data
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

#### Full Custom Configuration

```rust
use std::net::Ipv4Addr;
use simple_discover::{DiscoverConfig, DiscoverServer};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    // Create custom configuration
    let config = DiscoverConfig::new()
        .set_multicast_addr(Ipv4Addr::new(224, 0, 0, 200))
        .set_ports(vec![20001, 20010, 20100]);
    
    // Set custom data on the server
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

### Client

Initiate device discovery requests to get all responding devices and their custom data in the LAN:

#### Basic Usage

```rust
use simple_discover::DiscoverClient;
use std::time::Duration;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    // Create client with default configuration
    let client = DiscoverClient::new();
    
    // Initiate discovery request with 5-second timeout
    let devices = client.discover(Duration::from_secs(5)).await.unwrap();
    
    println!("Found {} device(s):\n", devices.len());
    
    for (i, device) in devices.iter().enumerate() {
        println!("Device {}:", i + 1);
        println!("  IP: {}", device.ip);
        
        if let Some(data) = &device.data {
            println!("  Data: {}", serde_json::to_string_pretty(data).unwrap());
        } else {
            println!("  Data: <none>");
        }
        println!();
    }
}
```

Example output:
```
Found 2 device(s):

Device 1:
  IP: 192.168.1.100
  Data: {
    "name": "My Service",
    "version": "1.0.0",
    "capabilities": ["http", "grpc"]
  }

Device 2:
  IP: 192.168.1.101
  Data: <none>
```

#### Custom Configuration

```rust
use std::net::Ipv4Addr;
use std::time::Duration;
use simple_discover::{DiscoverConfig, DiscoverClient};

#[tokio::main]
async fn main() {
    env_logger::init();
    
    // Create custom configuration
    let config = DiscoverConfig::new()
        .set_multicast_addr(Ipv4Addr::new(224, 0, 0, 200))
        .set_ports(vec![20001, 20010, 20100]);
    
    let client = DiscoverClient::with_config(config);
    
    // Initiate discovery request
    let devices = client.discover(Duration::from_secs(5)).await.unwrap();
    
    for device in devices {
        println!("Found device: {} {:?}", device.ip, device.data);
    }
}
```

## 📖 API Reference

### Configuration

The `DiscoverConfig` struct contains the following configurable options:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `multicast_addr` | `Ipv4Addr` | `224.0.0.100` | Multicast address |
| `ports` | `Vec<u16>` | `[10001, 10010, 10100, 11000]` | List of listening ports |
| `listen_addr` | `Ipv4Addr` | `0.0.0.0` (UNSPECIFIED) | Custom listen address (use UNSPECIFIED for auto-select) |

Configure using builder pattern:

```rust
let config = DiscoverConfig::new()
    .set_multicast_addr(Ipv4Addr::new(224, 0, 0, 200))
    .set_ports(vec![20001, 20010, 20100, 21000])
    .set_listen_addr(Ipv4Addr::new(192, 168, 1, 100)); // Specify a particular IP to listen on
```

### Server Custom Data

`DiscoverServer` supports setting custom data to be carried in responses via the `with_custom_data()` method:

```rust
let server = DiscoverServer::new()
    .with_custom_data(serde_json::json!({"key": "value"}));
```

Or combined with configuration:

```rust
let config = DiscoverConfig::new()
    .set_multicast_addr(Ipv4Addr::new(224, 0, 0, 200))
    .set_ports(vec![20001, 20010, 20100]);

let server = DiscoverServer::with_config(config)
    .with_custom_data(serde_json::json!({"service": "my-app"}));
```

### Return Types

#### `DiscoveredDevice`

The struct returned after the client discovers devices:

```rust
pub struct DiscoveredDevice {
    /// Device IP address
    pub ip: Ipv4Addr,
    /// Custom data (if provided by the server)
    pub data: Option<serde_json::Value>,
}
```

### Custom Data Format

Custom data can be any type that can be serialized to JSON:

```rust
use serde_json::json;

// Simple object
let data1 = json!({"name": "service-a"});

// Nested object
let data2 = json!({
    "service": "api-gateway",
    "config": {
        "port": 8080,
        "protocol": "https"
    },
    "tags": ["production", "v1"]
});

// Array
let data3 = json!(["feature1", "feature2", "feature3"]);

// Primitive types
let data4 = json!("just a string");
let data5 = json!(42);
```

**Note**: If `custom_data` is `None`, the server response will not include the `data` field to reduce network overhead.

#### ⚠️ Important: Serialization Failures Cause Panic

The `with_custom_data()` method will **panic** if the data cannot be serialized to JSON. The following situations may cause serialization failures:

1. **Unsupported types**: Such as function pointers, raw pointers, etc.
2. **Circular references**: Circular references in data structures
3. **Special floating-point numbers**: `NaN`, `Infinity`, `-Infinity` (not supported by JSON standard)
4. **Incorrect Serialize implementation**: Custom types with faulty Serialize implementations

**Examples of cases that will panic:**

```rust
use simple_discover::DiscoverServer;

// ❌ Error example: Floating-point with NaN
let data = vec![f64::NAN, 1.0, 2.0];
let server = DiscoverServer::new().with_custom_data(data); // This will panic!

// ❌ Error example: Infinity
let data = f64::INFINITY;
let server = DiscoverServer::new().with_custom_data(data); // This will also panic!
```

**How to avoid panic:**

```rust
use simple_discover::DiscoverServer;
use serde_json;

// ✅ Safe approach 1: Use json! macro (checked at compile time)
let data = serde_json::json!({"value": 42});
let server = DiscoverServer::new().with_custom_data(data);

// ✅ Safe approach 2: Pre-validate data
let my_data = vec![1.0, 2.0, 3.0]; // Ensure no NaN or Infinity
if serde_json::to_value(&my_data).is_ok() {
    let server = DiscoverServer::new().with_custom_data(my_data);
} else {
    eprintln!("Data cannot be serialized to JSON");
}

// ✅ Safe approach 3: Wrap with Result (requires manual handling)
fn create_server_safe<T: serde::Serialize>(data: T) -> Result<DiscoverServer, String> {
    // Try serialization first for validation
    serde_json::to_value(&data)
        .map(|_| DiscoverServer::new().with_custom_data(data))
        .map_err(|e| format!("Serialization failed: {}", e))
}
```

**Recommended practices:**
- 🎯 **Prefer using `json!` macro**: Catches most errors at compile time
- 🎯 **Avoid using floating-point numbers**, or use integers instead
- 🎯 **Test custom structs**: Ensure all fields are serializable
- 🎯 **In production environments**: Consider pre-validation before calling `with_custom_data()`

## 🔧 Protocol Design

To solve UDP packet sticking and loss issues, simple-discover uses a custom application-layer protocol:

```
+--------+---------+------+--------+--------+----------+
| Magic  | Version | Type | Length | CRC32  | Payload  |
| 4 bytes| 1 byte  |1 byte|4 bytes |4 bytes | variable |
+--------+---------+------+--------+--------+----------+
```

- **Magic**: `0x53445343` ("SDSC:Simple Discover Service Code") - Protocol magic number for identifying valid packets
- **Version**: Protocol version number (currently 1)
- **Type**: Message type (0=Request, 1=Response)
- **Length**: Payload length in bytes
- **CRC32**: CRC32 checksum of the payload to ensure data integrity
- **Payload**: Actual data in JSON format

### Message Format

**Request message:**
```json
{"type":"request"}
```

**Response message:**
```json
{
  "type": "response",
  "data": {...}  // Optional, only exists when server configured with custom_data
}
```

## 🧪 Running Examples

Open two terminals in the project root directory:

```bash
# Terminal 1: Start the server
cargo run --example server

# Terminal 2: Run the client (in another terminal)
cargo run --example client
```

## 📝 Use Cases

- **Microservice Discovery**: Automatic discovery and registration of microservices in LAN
- **P2P Networks**: Node discovery in peer-to-peer applications
- **IoT Device Management**: Automatic discovery and configuration of IoT devices
- **Distributed Systems**: Dynamic joining and leaving of cluster nodes
- **Development Tools**: Automatic service connection in local development environments

## ⚠️ Caveats

1. **LAN Limitation**: Only applicable to local area network (LAN) environments, depends on UDP multicast support
2. **Firewall Configuration**: Ensure firewall allows multicast traffic and configured ports
3. **Cross-Subnet Limitation**: Multicast typically cannot cross different subnets
4. **Data Size**: Custom data should not be too large, recommended to keep within a few KB
5. **Security**: No encryption or authentication mechanism provided, only suitable for trusted network environments
6. **⚠️ Serialization Panic**: `with_custom_data()` will panic if data cannot be serialized to JSON. Avoid using special floating-point numbers like `NaN` and `Infinity`. Prefer using the `json!` macro

## 📄 License

This project is licensed under the MIT License.

## 🤖 AI-Assisted Development

This project was developed with AI assistance, using Alibaba Cloud's Tongyi Lingma for code generation, refactoring, and documentation writing.

## 🤝 Contributing

Issues and Pull Requests are welcome!
