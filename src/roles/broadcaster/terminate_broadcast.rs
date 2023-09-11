use crate::types::ClientsMap;
use std::net::SocketAddr;

pub async fn stop(addr: SocketAddr, room_name: String, channel_peer_map: ClientsMap) {
    let mut channels = channel_peer_map.lock().await;
    println!(
        "Broadcaster: {:?} terminated broadcast in room {:?}",
        addr, room_name
    );
    for i in channels.get(&room_name).unwrap().room_users.iter() {
        i.transmiter.close_channel();
        // i.transmiter.disconnect();
    }
    channels.remove(&room_name).unwrap();
}
