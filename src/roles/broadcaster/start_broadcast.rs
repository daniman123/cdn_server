use crate::{
    event_handlers::handle_track_event,
    helpers::peer_helpers::peer_processor,
    models::{BroadcasterMetaData, Room},
    types::{ClientsMap, Tx},
    utils::create_peer_connection,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::mpsc::Sender;
use webrtc::{
    peer_connection::{sdp::session_description::RTCSessionDescription, RTCPeerConnection},
    rtp_transceiver::rtp_codec::RTPCodecType,
    track::track_local::track_local_static_rtp::TrackLocalStaticRTP,
};

pub async fn start(
    desc: RTCSessionDescription,
    tx: Tx,
    addr: SocketAddr,
    room_name: String,
    channel_peer_map: ClientsMap,
) {
    let (local_track_chan_tx, mut local_track_chan_rx) =
        tokio::sync::mpsc::channel::<Arc<TrackLocalStaticRTP>>(128);

    let peer_connection = broadcaster_peer(desc, local_track_chan_tx, tx.clone()).await;
    let mut tracks = Vec::new();
    while tracks.len() < 2 {
        if let Some(track) = local_track_chan_rx.recv().await {
            tracks.push(track.clone())
        } else {
            break;
        }
    }

    let broadcaster_meta_data = BroadcasterMetaData::new(tx.clone(), peer_connection, tracks);

    let room = Room::new(broadcaster_meta_data);

    println!("Broadcaster: {:?} created room {:?}", addr, room_name);

    let mut channels = channel_peer_map.lock().await;
    channels.insert(room_name, room);
}

pub async fn broadcaster_peer(
    desc: RTCSessionDescription,
    local_track_chan_tx: Sender<Arc<TrackLocalStaticRTP>>,
    tx: Tx,
) -> Arc<RTCPeerConnection> {
    let peer_connection = create_peer_connection().await;
    peer_connection
        .add_transceiver_from_kind(RTPCodecType::Video, None)
        .await
        .unwrap();

    peer_connection
        .add_transceiver_from_kind(RTPCodecType::Audio, None)
        .await
        .unwrap();

    let pc = Arc::downgrade(&peer_connection);
    handle_track_event(peer_connection.clone(), pc, local_track_chan_tx).await;

    peer_processor(peer_connection.clone(), desc, tx.clone()).await
}
