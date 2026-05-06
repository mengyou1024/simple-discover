# Simple Discover

A lightweight service discovery library based on Rust, using UDP multicast to implement automatic device discovery within local area networks. Supports custom data transmission and reliable application-layer protocols.

## ✨ Features

- 🔍 **Automatic Discovery**: Zero configuration, automatically discovers devices in the LAN
- 📦 **Custom Data**: Servers can carry arbitrary JSON-formatted custom data in responses
- 🔄 **Multi-port Fault Tolerance**: Supports multiple ports (10001, 10010, 10100, 11000) to prevent port occupation issues
- ✅ **Data Validation**: Uses CRC32 checksums to ensure data integrity
- 📋 **Reliable Protocol**: Custom application-layer protocol to solve UDP packet sticking and loss problems
- ⚙️ **Highly Configurable**: Supports custom multicast addresses, port lists, and metadata

## 📦 Installation

Add dependencies to `Cargo.toml`:

```toml
[dependencies]
simple-discover = "*"
serde_json = "*"
tokio = { version = "*", features = ["full"] }
```

## 🚀 Quick Start

### Basic Examples

#### Server

The simplest server implementation:

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

#### Client

The simplest client implementation:

```rust
use simple_discover::DiscoverClient;
use std::time::Duration;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    let client = DiscoverClient::new();
    let devices = client.discover(Duration::from_secs(5)).await.unwrap();
    
    println!("Discovered {} devices:\n", devices.len());
    for (i, device) in devices.iter().enumerate() {
        println!("Device {}: IP = {}", i + 1, device.ip);
    }
}
```

### Advanced Usage

#### Server with Custom Data

The server can carry custom metadata in responses (such as service name, version, capabilities, etc.):

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

#### Full Custom Configuration

Customize both network configuration and service data:

```rust
use std::net::Ipv4Addr;
use simple_discover::{DiscoverConfig, DiscoverServer};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    // Custom network configuration
    let config = DiscoverConfig::new()
        .set_multicast_addr(Ipv4Addr::new(224, 0, 0, 200))
        .set_ports(vec![20001, 20010, 20100])
        .set_listen_addr(Ipv4Addr::new(192, 168, 1, 100));
    
    // Create server with custom data
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

#### Client Receiving Custom Data

The client can retrieve custom data returned by the server:

```rust
use simple_discover::DiscoverClient;
use std::time::Duration;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    let client = DiscoverClient::new();
    let devices = client.discover(Duration::from_secs(5)).await.unwrap();
    
    println!("Discovered {} devices:\n", devices.len());
    
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
Discovered 2 devices:

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

## 📖 API Reference

### Configuration Options

The `DiscoverConfig` struct contains the following configurable items:

| Field | Type | Default Value | Description |
|-------|------|---------------|-------------|
| `multicast_addr` | `Ipv4Addr` | `224.0.0.100` | Multicast address |
| `ports` | `Vec<u16>` | `[10001, 10010, 10100, 11000]` | List of listening ports |
| `listen_addr` | `Ipv4Addr` | `0.0.0.0` (UNSPECIFIED) | Custom listen address (automatically selected when set to UNSPECIFIED) |

Configure using builder pattern:

```rust
let config = DiscoverConfig::new()
    .set_multicast_addr(Ipv4Addr::new(224, 0, 0, 200))
    .set_ports(vec![20001, 20010, 20100, 21000])
    .set_listen_addr(Ipv4Addr::new(192, 168, 1, 100));
```

### Server API

#### Creating a Server

```rust
// Using default configuration
let server = DiscoverServer::new();

// Using custom configuration
let config = DiscoverConfig::new()
    .set_multicast_addr(Ipv4Addr::new(224, 0, 0, 200));
let server = DiscoverServer::with_config(config);
```

#### Setting Custom Data

```rust
let server = DiscoverServer::new()
    .with_custom_data(serde_json::json!({"key": "value"}));
```

Or combined with configuration:

```rust
let server = DiscoverServer::with_config(config)
    .with_custom_data(serde_json::json!({"service": "my-app"}));
```

#### Starting and Stopping

```rust
// Start the server
let handle = server.start().await?;

// Stop the server
server.stop();
let _ = handle.await;
```

### Client API

#### Creating a Client

```rust
// Using default configuration
let client = DiscoverClient::new();

// Using custom configuration
let config = DiscoverConfig::new()
    .set_multicast_addr(Ipv4Addr::new(224, 0, 0, 200));
let client = DiscoverClient::with_config(config);
```

#### Initiating Discovery Requests

```rust
let devices = client.discover(Duration::from_secs(5)).await?;
```

### Data Structures

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

**Notes**:
- If `custom_data` is `None`, the Payload in the server response will be empty
- The Payload is directly the JSON serialization result of `custom_data`

## 🔧 Protocol Design

To solve UDP packet sticking and loss problems, simple-discover uses a custom application-layer protocol:

### Protocol Header Structure

```
+--------+------+--------+--------+----------+
| Magic  | Type | Length | CRC32  | Payload  |
| 4 bytes|1 byte|4 bytes |4 bytes | variable |
+--------+------+--------+--------+----------+
```

### Field Descriptions

- **Magic**: `0x53445343` ("SDSC:Simple Discover Service Code") - Protocol magic number for identifying valid packets
- **Type**: Message type (0=Request, 1=Response)
- **Length**: Payload length (in bytes)
- **CRC32**: CRC32 checksum of the Payload to ensure data integrity
- **Payload**: Actual data in JSON format (optional)

### Message Format

#### Request Message

- **Purpose**: Client initiates device discovery request
- **Payload**: Empty (length is 0)

```
Header: Magic + Type(0) + Length(0) + CRC32(empty)
Payload: (empty)
```

#### Response Message

- **Purpose**: Server responds to discovery request
- **Payload**:
  - If `custom_data` is configured: Directly the JSON serialization result of `custom_data`
  - If `custom_data` is not configured: Empty (length is 0)

Response with custom data:
```
Header: Magic + Type(1) + Length(N) + CRC32(data)
Payload: {"name": "My Service", "version": "1.0.0"}
```

Response without custom data:
```
Header: Magic + Type(1) + Length(0) + CRC32(empty)
Payload: (empty)
```

### Design Principles

1. **Simplicity**: Removed redundant Version and type fields to reduce protocol overhead
2. **Separation of Concerns**: Message type is managed by the Type field in the Header, Payload only carries business data
3. **Flexibility**: Allows servers to send arbitrary JSON structures directly without additional wrapping
4. **Reliability**: Ensures data integrity through CRC32 checksums

## 🧪 Running Examples

Open two terminals in the project root directory:

```bash
# Terminal 1: Start the server
cargo run --example server

# Terminal 2: Run the client (in another terminal)
cargo run --example client
```

## 📝 Use Cases

- **Microservice Discovery**: Automatic discovery and registration of microservices within LAN
- **P2P Networks**: Node discovery in peer-to-peer applications
- **IoT Device Management**: Automatic discovery and configuration of IoT devices
- **Distributed Systems**: Dynamic joining and leaving of cluster nodes
- **Development Tools**: Automatic service connection in local development environments

## ⚠️ Considerations

1. **LAN Limitation**: Only applicable to LAN environments, relies on UDP multicast support
2. **Firewall Configuration**: Ensure firewalls allow multicast traffic and configured ports
3. **Cross-subnet Limitation**: Multicast typically cannot cross different subnets
4. **Data Size**: Custom data should not be too large, recommended to keep within a few KB
5. **Security**: No encryption or authentication mechanisms provided, only suitable for trusted network environments
6. **Serialization Requirements**: `with_custom_data()` requires data to be successfully serializable to JSON, avoid using special floating-point numbers (such as `NaN`, `Infinity`)

## 📄 License

This project is licensed under the MIT License.

## 🤖 AI-Assisted Development

This project was developed with AI assistance, using Alibaba Cloud Tongyi Lingma for code generation, refactoring, and documentation writing.

## 🤝 Contributing

Issues and Pull Requests are welcome!
