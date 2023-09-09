// use tokio::sync::mpsc::Receiver;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use webrtc::{
    peer_connection::sdp::session_description::RTCSessionDescription,
    track::track_local::track_local_static_rtp::TrackLocalStaticRTP,
};

use crate::types::{PeerConn, Tx};

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

#[derive(Debug)]
pub struct Room {
    pub room_users: Vec<ViewerMetaData>,
    pub broadcaster: BroadcasterMetaData,
    // pub broadcast: Vec<RTCRtpCodecCapability>,
}

#[derive(Debug)]
pub struct BroadcasterMetaData {
    pub transmiter: Tx,
    pub broadcaster_peer: PeerConn,
    pub track_channel_rx: Arc<TrackLocalStaticRTP>,
    // pub track_channel_rx: Receiver<Arc<TrackLocalStaticRTP>>,
}

#[derive(Debug)]
pub struct ViewerMetaData {
    pub transmiter: Tx,
    pub viewer_peer: PeerConn,
}
