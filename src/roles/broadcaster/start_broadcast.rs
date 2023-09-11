use crate::{
    event_handlers::{handle_candidate_event, handle_track_event},
    models::{BroadcasterMetaData, Room},
    types::{ClientsMap, Tx},
    utils::create_peer_connection,
};
use serde_json::json;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::tungstenite::Message;
use webrtc::{
    peer_connection::{
        peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription, RTCPeerConnection,
    },
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
            // println!("{:?}", track);
            tracks.push(track.clone())
        } else {
            break;
        }
    }
    // println!("tracks {:?}", tracks);
    let broadcaster_meta_data = BroadcasterMetaData {
        transmiter: tx.clone(),
        broadcaster_peer: peer_connection,
        track_channel_rx: tracks,
    };

    let room = Room {
        broadcaster: broadcaster_meta_data,
        room_users: vec![],
    };
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

    peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
        println!("Peer Connection State has changed: {s}");
        Box::pin(async {})
    }));

    let mut gathering_complete_rx = handle_candidate_event(peer_connection.clone()).await;

    peer_connection.set_remote_description(desc).await.unwrap();
    let answer = peer_connection.create_answer(None).await.unwrap();

    // Sets the LocalDescription, and starts our UDP listeners
    peer_connection
        .set_local_description(answer.clone())
        .await
        .unwrap();

    let cand_recv = gathering_complete_rx.recv().await.unwrap();
    peer_connection.add_ice_candidate(cand_recv).await.unwrap();

    let local_desc = peer_connection.local_description().await.unwrap();

    let payload = json!(local_desc);
    let answer = Message::Text(payload.to_string());

    if let Err(err) = tx.unbounded_send(answer.clone()) {
        eprintln!("Failed to send message to recipient: {} {:?}", err, tx);
    }

    peer_connection
}
