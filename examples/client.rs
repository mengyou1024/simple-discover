use std::time::Duration;

use simple_discover::DiscoverClient;

#[tokio::main]
async fn main() {
    env_logger::init();

    // 使用默认配置
    let client = DiscoverClient::new();
    let devices = client.discover(Duration::from_secs(5)).await.unwrap();

    // 或者使用自定义配置
    // use std::net::Ipv4Addr;
    // use simple_discover::DiscoverConfig;

    // let config = DiscoverConfig::new()
    //     .set_multicast_addr(Ipv4Addr::new(224, 0, 0, 200))
    //     .set_ports(vec![20001, 20010, 20100])
    //     .set_listen_addr(Ipv4Addr::new(192, 168, 1, 50)); // 设置自定义监听地址
    // let client = DiscoverClient::with_config(config);
    // let devices = client.discover(Duration::from_secs(5)).await.unwrap();

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
