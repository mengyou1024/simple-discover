use serde::Serialize;
use serde_json::json;
use simple_discover::DiscoverServer;
use tokio::signal;

// 自定义结构体示例
#[derive(Serialize)]
#[allow(dead_code)]
struct ServiceInfo {
    name: String,
    version: String,
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // 示例 1: 使用 json! 宏（最常用）
    let custom_data = json!({
        "name": "My Server",
        "version": "1.0.0",
        "services": ["http", "grpc"],
        "metadata": {
            "region": "cn-north-1",
            "environment": "production"
        }
    });
    let server = DiscoverServer::new().with_custom_data(custom_data);

    // 示例 2: 使用自定义结构体（自动序列化）
    // let service_info = ServiceInfo {
    //     name: "My Service".to_string(),
    //     version: "2.0.0".to_string(),
    //     port: 8080,
    // };
    // let server = DiscoverServer::new().with_custom_data(service_info);

    // 示例 3: 使用 HashMap
    // use std::collections::HashMap;
    // let mut map = HashMap::new();
    // map.insert("key1", "value1");
    // map.insert("key2", "value2");
    // let server = DiscoverServer::new().with_custom_data(map);

    // 示例 4: 使用 Vec
    // let tags = vec!["tag1", "tag2", "tag3"];
    // let server = DiscoverServer::new().with_custom_data(tags);

    // 示例 5: 不使用自定义数据
    // let server = DiscoverServer::new();

    // 或者使用完整的自定义配置
    // use std::net::Ipv4Addr;
    // use simple_discover::{DiscoverConfig, DiscoverServer};
    //
    // let config = DiscoverConfig::new()
    //     .set_multicast_addr(Ipv4Addr::new(224, 0, 0, 200))
    //     .set_ports(vec![20001, 20010, 20100])
    //     .set_listen_addr(Ipv4Addr::new(192, 168, 1, 100)); // 设置自定义监听地址
    // let server = DiscoverServer::with_config(config)
    //     .with_custom_data(json!({"service": "my-app"}));

    let join_handle = server.start().await?;

    println!("Server is running with custom data. Press Ctrl+C to stop.");

    // 等待 Ctrl+C 信号
    signal::ctrl_c().await?;

    println!("\nReceived Ctrl+C, shutting down...");

    server.stop();

    // 等待服务器任务完成
    let _ = join_handle.await;

    println!("Server stopped gracefully.");

    Ok(())
}
