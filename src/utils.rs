use std::sync::Arc;
use webrtc::{
    ice_transport::ice_server::RTCIceServer,
    peer_connection::{configuration::RTCConfiguration, RTCPeerConnection},
};

use crate::helpers::build_api;
// "stun:stun1.1.google.com:19302", "stun:stun2.1.google.com:19302"
pub fn rtc_config() -> RTCConfiguration {
    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {

            urls: vec!["stun:stun1.1.google.com:19302".to_owned(), "stun:stun2.1.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };
    config
}

pub async fn create_peer_connection() -> Arc<RTCPeerConnection> {
    let api = build_api();
    let config = rtc_config();

    Arc::new(api.new_peer_connection(config).await.unwrap())
}
