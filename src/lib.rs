use std::fmt;
use std::io;
use std::net::Ipv4Addr;

/// 客户端
mod client;
/// 服务器
mod server;

/// 默认组播地址
const DEFAULT_MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 100);
/// 默认设备发现可用的端口(使用多个端口, 防止端口被某一程序占用)
const DEFAULT_DISCOVER_PORT: &[u16] = &[10001, 10010, 10100, 11000];

/// 协议魔数 "SDSC" (Simple Discover Service Code)
const PROTOCOL_MAGIC: u32 = 0x53445343;
/// 消息类型：请求
const MSG_TYPE_REQUEST: u8 = 0;
/// 消息类型：响应
const MSG_TYPE_RESPONSE: u8 = 1;
/// 协议头部长度：Magic(4) + Type(1) + Length(4) + CRC32(4) = 13 bytes
const PROTOCOL_HEADER_SIZE: usize = 13;
/// 协议缓冲区大小
const PROTOCOL_BUFFER_SIZE: usize = 4096;

/// 发现配置
#[derive(Debug, Clone)]
pub struct DiscoverConfig {
    /// 组播地址
    pub multicast_addr: Ipv4Addr,
    /// 可用的端口列表
    pub ports: Vec<u16>,
    /// 自定义监听地址（可选），如果设置则使用此地址而不是 0.0.0.0
    pub listen_addr: Ipv4Addr,
}

impl Default for DiscoverConfig {
    fn default() -> Self {
        Self {
            multicast_addr: DEFAULT_MULTICAST_ADDR,
            ports: DEFAULT_DISCOVER_PORT.to_vec(),
            listen_addr: Ipv4Addr::UNSPECIFIED,
        }
    }
}

impl DiscoverConfig {
    /// 创建一个新的配置，使用默认值
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置组播地址
    pub fn set_multicast_addr<A: Into<Ipv4Addr>>(mut self, addr: A) -> Self {
        self.multicast_addr = addr.into();
        self
    }

    /// 设置端口列表
    pub fn set_ports<P: Into<Vec<u16>>>(mut self, ports: P) -> Self {
        self.ports = ports.into();
        self
    }

    /// 设置自定义监听地址
    pub fn set_listen_addr<A: Into<Ipv4Addr>>(mut self, addr: A) -> Self {
        self.listen_addr = addr.into();
        self
    }
}

/// 自定义错误类型
#[derive(Debug)]
pub enum DiscoverError {
    /// IO 错误
    Io(io::Error),
    /// 没有可用端口
    NoAvailablePort,
    /// 协议错误
    Protocol(String),
    /// 其他错误
    Other(String),
}

impl fmt::Display for DiscoverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiscoverError::Io(err) => write!(f, "{}", err),
            DiscoverError::NoAvailablePort => write!(f, "No available port"),
            DiscoverError::Protocol(msg) => write!(f, "Protocol error: {}", msg),
            DiscoverError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for DiscoverError {}

impl From<io::Error> for DiscoverError {
    fn from(err: io::Error) -> Self {
        DiscoverError::Io(err)
    }
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, DiscoverError>;

/// 发现的设备信息
#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    /// 设备 IP 地址
    pub ip: Ipv4Addr,
    /// 自定义数据（如果服务器提供了的话）
    pub data: Option<serde_json::Value>,
}

/// 计算 CRC32 校验码
fn calculate_crc32(data: &[u8]) -> u32 {
    crc32fast::hash(data)
}

/// 编码请求消息
fn encode_request() -> Vec<u8> {
    // 请求消息没有payload
    let payload = &[];
    let crc = calculate_crc32(payload);

    let mut buffer = Vec::with_capacity(PROTOCOL_HEADER_SIZE + payload.len());

    // Magic (4 bytes, little-endian)
    buffer.extend_from_slice(&PROTOCOL_MAGIC.to_le_bytes());
    // Type (1 byte): 0 = Request
    buffer.push(MSG_TYPE_REQUEST);
    // Length (4 bytes, little-endian)
    buffer.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    // CRC32 (4 bytes, little-endian)
    buffer.extend_from_slice(&crc.to_le_bytes());
    // Payload (empty for request)
    buffer.extend_from_slice(payload);

    buffer
}

/// 编码响应消息
fn encode_response(custom_data: Option<&serde_json::Value>) -> crate::Result<Vec<u8>> {
    let payload = if let Some(data) = custom_data {
        // 如果有自定义数据，直接使用它作为payload
        serde_json::to_vec(data)
            .map_err(|e| DiscoverError::Protocol(format!("Failed to serialize custom data: {}", e)))?
    } else {
        // 如果没有自定义数据，payload为空
        vec![]
    };

    let crc = calculate_crc32(&payload);

    let mut buffer = Vec::with_capacity(PROTOCOL_HEADER_SIZE + payload.len());

    // Magic (4 bytes, little-endian)
    buffer.extend_from_slice(&PROTOCOL_MAGIC.to_le_bytes());
    // Type (1 byte): 1 = Response
    buffer.push(MSG_TYPE_RESPONSE);
    // Length (4 bytes, little-endian)
    buffer.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    // CRC32 (4 bytes, little-endian)
    buffer.extend_from_slice(&crc.to_le_bytes());
    // Payload
    buffer.extend_from_slice(&payload);

    Ok(buffer)
}

/// 解码消息
/// # Returns
/// (msg_type, payload_json) - 消息类型和payload的JSON值
/// 如果payload为空，返回None
fn decode_message(buffer: &[u8]) -> crate::Result<(u8, Option<serde_json::Value>)> {
    if buffer.len() < PROTOCOL_HEADER_SIZE {
        return Err(DiscoverError::Protocol(format!(
            "Buffer too short: {} bytes, expected at least {}",
            buffer.len(),
            PROTOCOL_HEADER_SIZE
        )));
    }

    // Parse header
    let magic = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
    if magic != PROTOCOL_MAGIC {
        return Err(DiscoverError::Protocol(format!(
            "Invalid magic: 0x{:08X}",
            magic
        )));
    }

    let msg_type = buffer[4];
    let length = u32::from_le_bytes([buffer[5], buffer[6], buffer[7], buffer[8]]) as usize;
    let expected_crc = u32::from_le_bytes([buffer[9], buffer[10], buffer[11], buffer[12]]);

    if buffer.len() < PROTOCOL_HEADER_SIZE + length {
        return Err(DiscoverError::Protocol(format!(
            "Incomplete message: buffer has {} bytes, payload needs {} bytes",
            buffer.len() - PROTOCOL_HEADER_SIZE,
            length
        )));
    }

    let payload = &buffer[PROTOCOL_HEADER_SIZE..PROTOCOL_HEADER_SIZE + length];

    // Verify CRC32
    let actual_crc = calculate_crc32(payload);
    if actual_crc != expected_crc {
        return Err(DiscoverError::Protocol(format!(
            "CRC32 mismatch: expected 0x{:08X}, got 0x{:08X}",
            expected_crc, actual_crc
        )));
    }

    // Parse JSON payload (if not empty)
    let json = if payload.is_empty() {
        None
    } else {
        Some(serde_json::from_slice(payload)
            .map_err(|e| DiscoverError::Protocol(format!("Failed to parse JSON: {}", e)))?)
    };

    Ok((msg_type, json))
}

pub use client::DiscoverClient;
pub use server::DiscoverServer;

#[cfg(test)]
mod tests;
