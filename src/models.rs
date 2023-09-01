use serde::{Deserialize, Serialize};

use std::net::SocketAddr;
use webrtc::{
    peer_connection::sdp::session_description::RTCSessionDescription,
    rtp_transceiver::rtp_codec::RTCRtpCodecParameters,
};

use crate::types::Tx;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ViewerPayload {
    pub eventType: String,
    pub roomName: String,
    pub localDesc: Option<RTCSessionDescription>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct BroadCastPayload {
    pub eventType: String,
    pub roomName: String,
    pub localDesc: Option<RTCSessionDescription>,
}

#[derive(Clone, Debug)]
pub struct Room {
    pub room_users: Vec<(SocketAddr, Tx)>,
    pub broadcaster: (SocketAddr, Tx),
    pub broadcast: RTCRtpCodecParameters,
}

impl Room {
    pub fn create_room(broadcaster: (SocketAddr, Tx)) -> Self {
        Self {
            room_users: vec![],
            broadcaster,
            broadcast: RTCRtpCodecParameters::default(),
        }
    }
}
