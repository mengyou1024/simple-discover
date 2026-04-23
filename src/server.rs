use std::net::{Ipv4Addr, SocketAddrV4};

use tokio::{net::UdpSocket, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::{DiscoverConfig, DiscoverError, MSG_TYPE_REQUEST, decode_message, encode_response};

/// 获取可用的套接字
/// # Arguments
/// * ports: 可用的端口
/// * listen_addr: 自定义监听地址（可选）
/// # Returns
/// 可用的套接字
async fn get_available_socket(ports: &[u16], listen_addr: Ipv4Addr) -> crate::Result<UdpSocket> {
    // 否则尝试所有端口
    for port in ports {
        if let Ok(socket) = UdpSocket::bind(SocketAddrV4::new(listen_addr, *port)).await {
            return Ok(socket);
        } else {
            log::warn!("Port {} is not available", port);
        }
    }
    Err(DiscoverError::NoAvailablePort)
}

/// 处理单个发现请求
/// # Arguments
/// * `socket` - UDP 套接字
/// * `src` - 请求来源地址
/// * `received_data` - 接收到的数据
/// * `multicast_addr` - 组播地址（用于日志）
/// * `custom_data` - 自定义数据
async fn handle_discover_request(
    socket: &UdpSocket,
    src: std::net::SocketAddr,
    received_data: &[u8],
    multicast_addr: Ipv4Addr,
    custom_data: Option<&serde_json::Value>,
) {
    // 尝试解码消息
    match decode_message(received_data) {
        Ok((msg_type, _json)) => {
            if msg_type == MSG_TYPE_REQUEST {
                log::info!("recv request from {} at multicast {}", src, multicast_addr);

                // 编码响应
                match encode_response(custom_data) {
                    Ok(response_buf) => match socket.send_to(&response_buf, src).await {
                        Ok(_) => log::info!("send response to {}", src),
                        Err(err) => log::error!("send error: {}", err),
                    },
                    Err(err) => log::error!("encode response error: {}", err),
                }
            }
        }
        Err(err) => {
            log::warn!("decode message error from {}: {}", src, err);
        }
    }
}

/// 发现服务器
pub struct DiscoverServer {
    cancellation_token: CancellationToken,
    config: DiscoverConfig,
    custom_data: Option<serde_json::Value>,
}

impl Drop for DiscoverServer {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}

impl Default for DiscoverServer {
    fn default() -> Self {
        Self::new()
    }
}

impl DiscoverServer {
    /// 创建一个发现服务，使用默认配置
    pub fn new() -> Self {
        Self::with_config(DiscoverConfig::default())
    }

    /// 创建一个发现服务，使用自定义配置
    pub fn with_config(config: DiscoverConfig) -> Self {
        Self {
            cancellation_token: CancellationToken::new(),
            config,
            custom_data: None,
        }
    }

    /// 设置自定义数据（便捷方法）
    /// # Arguments
    /// * `data` - 实现了 Serialize trait 的数据
    /// # Panics
    /// 如果数据无法序列化为 JSON，将会 panic
    pub fn with_custom_data<T: serde::Serialize>(mut self, data: T) -> Self {
        match serde_json::to_value(data) {
            Ok(value) => self.custom_data = Some(value),
            Err(err) => panic!("Failed to serialize custom data to JSON: {}", err),
        }
        self
    }

    /// 启动服务
    pub async fn start(&self) -> crate::Result<JoinHandle<()>> {
        let socket = get_available_socket(&self.config.ports, self.config.listen_addr).await?;

        // 加入组播
        socket.join_multicast_v4(self.config.multicast_addr, Ipv4Addr::UNSPECIFIED)?;

        log::info!("start discover server: {}", socket.local_addr()?);

        let cancellation_token = self.cancellation_token.clone();
        let multicast_addr = self.config.multicast_addr;
        let custom_data = self.custom_data.clone();

        Ok(tokio::spawn(async move {
            // 使用较大的缓冲区以支持自定义数据
            let mut buf = [0u8; 4096];
            loop {
                tokio::select! {
                    _ = cancellation_token.cancelled() => break,
                    result = socket.recv_from(&mut buf) => {
                        if let Ok((len, src)) = result {
                            // 处理发现请求
                            handle_discover_request(
                                &socket,
                                src,
                                &buf[..len],
                                multicast_addr,
                                custom_data.as_ref(),
                            ).await;
                        }
                    }
                };
            }
            log::info!("stop discover server");
        }))
    }

    /// 停止服务
    pub fn stop(self) {
        drop(self);
    }
}
