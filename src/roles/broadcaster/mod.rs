mod start_broadcast;
mod terminate_broadcast;

use crate::{
    models::BroadCastPayload,
    types::{ClientsMap, Tx},
};
use std::net::SocketAddr;

pub async fn process_broadcaster(
    parsed_payload: BroadCastPayload,
    tx: Tx,
    addr: SocketAddr,
    channel_peer_map: ClientsMap,
) {
    // Handle Broadcaster payload
    let room_name = parsed_payload.roomName;
    let event_type = parsed_payload.eventType;

    if event_type == "START_BROADCAST" {
        if let Some(desc) = parsed_payload.localDesc {
            start_broadcast::start(desc, tx, addr, room_name.clone(), channel_peer_map.clone())
                .await;
        }
    }
    if event_type == "TERMINATE_BROADCAST" {
        terminate_broadcast::stop(addr, room_name.clone(), channel_peer_map.clone()).await;
    }
}
