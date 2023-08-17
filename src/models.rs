use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use futures_util::lock::Mutex;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecParameters;

pub struct RoomContainer {
    container: HashMap<RTCRtpCodecParameters, Vec<SocketAddr>>,
}

pub struct RoomBroadcastContainer {
    room: HashMap<String, RoomContainer>,
}

pub struct BroadcastRoomMap {
    room_maps: Arc<Mutex<RoomBroadcastContainer>>,
}

impl RoomBroadcastContainer {
    pub fn get_room(&self, room_name: String) -> Option<(&String, &RoomContainer)> {
        self.room.get_key_value(&room_name)
    }

    pub fn add_user(&self, room_name: String, addr: SocketAddr) {
        self.room.get_mut(&room_name).unwrap().container.raw_entry_mut(codec).
    }
}
