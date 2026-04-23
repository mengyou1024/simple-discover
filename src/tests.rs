use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::time::timeout;

use super::*;

async fn test_discover_default() {
    let server = DiscoverServer::new();
    let client = DiscoverClient::new();

    tokio::select! {
        Err(_) = timeout(Duration::from_secs(10), server.start()) => panic!("discover timeout"),
        devices = client.discover(Duration::from_secs(3)) => {
            assert!(devices.is_ok());
            assert!(devices.unwrap().len() > 0);
        }
    }
    server.stop();
}

async fn test_discover_custom_data() {
    let server = DiscoverServer::new().with_custom_data(0);
    let client = DiscoverClient::new();

    tokio::select! {
        Err(_) = timeout(Duration::from_secs(10), server.start()) => panic!("discover timeout"),
        devices = client.discover(Duration::from_secs(3)) => {
            assert!(devices.is_ok());
            assert!(devices.as_ref().unwrap().len() > 0);
            let device = devices.as_ref().unwrap()[0].clone();
            assert_eq!(device.data.unwrap().as_i64().unwrap(), 0);
        }
    }
    server.stop();
}

#[derive(Serialize, Deserialize)]
pub struct CustomData {
    name: String,
    version: String,
}

async fn test_discover_custom_struct() {
    let server = DiscoverServer::new().with_custom_data(CustomData {
        name: "My Server".to_string(),
        version: "1.0.0".to_string(),
    });
    let client = DiscoverClient::new();

    tokio::select! {
        Err(_) = timeout(Duration::from_secs(10), server.start()) => panic!("discover timeout"),
        devices = client.discover(Duration::from_secs(3)) => {
            assert!(devices.is_ok());
            assert!(devices.as_ref().unwrap().len() > 0);
            let device = devices.as_ref().unwrap()[0].clone();
            assert_eq!(device.data.as_ref().unwrap().get("name").unwrap().as_str().unwrap(), "My Server");
            assert_eq!(device.data.as_ref().unwrap().get("version").unwrap().as_str().unwrap(), "1.0.0");
        }
    }
}

async fn test_discover_big_data() {
    let buffer = include_str!("../README.md");
    let server = DiscoverServer::new().with_custom_data(buffer);
    let client = DiscoverClient::new();

    tokio::select! {
        Err(_) = timeout(Duration::from_secs(10), server.start()) => panic!("discover timeout"),
        devices = client.discover(Duration::from_secs(3)) => {
            assert!(devices.is_ok());
            assert!(devices.as_ref().unwrap().len() > 0);
        }
    }
    server.stop();
}

#[tokio::test]
#[should_panic(expected = "Custom data is too large")]
async fn test_discover_serial() {
    test_discover_default().await;
    test_discover_custom_data().await;
    test_discover_custom_struct().await;
    test_discover_big_data().await;
}
