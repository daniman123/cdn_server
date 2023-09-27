use crate::types::{PeerConn, TrackChannel, Tx};
use serde::{Deserialize, Serialize};
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

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
}

impl Room {
    pub fn new(broadcaster: BroadcasterMetaData) -> Room {
        Room {
            room_users: vec![],
            broadcaster,
        }
    }
}

#[derive(Debug)]
pub struct BroadcasterMetaData {
    pub transmiter: Tx,
    pub broadcaster_peer: PeerConn,
    pub track_channel_rx: TrackChannel,
}

impl BroadcasterMetaData {
    pub fn new(
        transmiter: Tx,
        broadcaster_peer: PeerConn,
        track_channel_rx: TrackChannel,
    ) -> BroadcasterMetaData {
        BroadcasterMetaData {
            transmiter,
            broadcaster_peer,
            track_channel_rx,
        }
    }
}

#[derive(Debug)]
pub struct ViewerMetaData {
    pub transmiter: Tx,
    pub viewer_peer: PeerConn,
}
