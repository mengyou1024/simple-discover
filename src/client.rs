use std::{
    net::{Ipv4Addr, SocketAddrV4},
    time::Duration,
};

use tokio::{
    net::UdpSocket,
    time::{Instant, timeout_at},
};

use crate::{DiscoverConfig, DiscoveredDevice, MSG_TYPE_RESPONSE, decode_message, encode_request};

/// 设备发现客户端
pub struct DiscoverClient {
    config: DiscoverConfig,
}

impl Default for DiscoverClient {
    fn default() -> Self {
        Self::new()
    }
}

impl DiscoverClient {
    /// 创建一个发现客户端，使用默认配置
    pub fn new() -> Self {
        Self::with_config(DiscoverConfig::default())
    }

    /// 创建一个发现客户端，使用自定义配置
    pub fn with_config(config: DiscoverConfig) -> Self {
        Self { config }
    }

    /// 启动设备发现
    /// # Arguments
    /// * `timeout` - 设备发现超时时间
    /// # Returns
    /// `Vec<DiscoveredDevice>` - 发现到的设备列表（包含 IP 和可选的自定义数据）
    pub async fn discover(&self, timeout: Duration) -> crate::Result<Vec<DiscoveredDevice>> {
        let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).await?;

        log::info!("start discovering at {}", socket.local_addr()?);

        // 加入组播
        socket.join_multicast_v4(self.config.multicast_addr, Ipv4Addr::UNSPECIFIED)?;

        let instant = Instant::now();

        // 使用较大的缓冲区以支持自定义数据
        let mut buf = [0u8; 4096];

        let mut devices = vec![];

        // 编码请求消息
        let request_buf = encode_request();

        // 向所有可用端口发送请求
        for port in &self.config.ports {
            let multiaddr = SocketAddrV4::new(self.config.multicast_addr, *port);
            socket.send_to(&request_buf, multiaddr).await?;
        }

        log::info!(
            "send request to {}:[{:?}]",
            self.config.multicast_addr,
            self.config.ports
        );

        loop {
            match timeout_at(instant + timeout, socket.recv_from(&mut buf)).await {
                Ok(Ok((len, src))) => {
                    let received_data = &buf[..len];

                    // 尝试解码消息
                    if let Ok((msg_type, json)) = decode_message(received_data) {
                        if msg_type == MSG_TYPE_RESPONSE {
                            // 从源地址提取 IP
                            if let std::net::SocketAddr::V4(src_addr) = src {
                                let ip = *src_addr.ip();

                                // 解析响应 JSON，提取自定义数据（如果存在）
                                let data =
                                    json.as_object().and_then(|obj| obj.get("data")).cloned();

                                log::info!(
                                    "found device at {} with data: {}",
                                    ip,
                                    data.as_ref()
                                        .map(|data| serde_json::to_string_pretty(data)
                                            .unwrap_or("<none>".to_string()))
                                        .unwrap_or_default()
                                );

                                devices.push(DiscoveredDevice { ip, data });
                            }
                        }
                    } else {
                        log::warn!("received invalid message from {}", src);
                    }
                }
                Ok(Err(err)) => log::error!("recv error: {}", err),
                Err(_) => break,
            }
        }

        // 去重（基于 IP）
        devices.sort_by_key(|a| a.ip);
        devices.dedup_by(|a, b| a.ip == b.ip);

        Ok(devices)
    }
}
