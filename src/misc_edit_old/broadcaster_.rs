use serde_json::{json, Map, Value};
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

use crate::{
    event_handlers::{handle_candidate_event, handle_track_event},
    models::{BroadCastPayload, BroadcasterMetaData, Room},
    types::{ClientsMap, Tx},
    utils::create_peer_connection,
};

pub async fn process_broadcaster(
    raw_payload: &Map<String, Value>,
    tx: Tx,
    addr: SocketAddr,
    channel_peer_map: ClientsMap,
) {
    // Handle Broadcaster payload
    if let Ok(parsed_payload) =
        serde_json::from_value::<BroadCastPayload>(Value::Object(raw_payload.clone()))
    {
        let room_name = parsed_payload.roomName;
        let event_type = parsed_payload.eventType;
        if event_type == "START_BROADCAST" {
            if let Some(desc) = parsed_payload.localDesc {
                let (local_track_chan_tx, mut local_track_chan_rx) =
                    tokio::sync::mpsc::channel::<Arc<TrackLocalStaticRTP>>(128);

                let peer_connection = broadcaster_peer(desc, local_track_chan_tx, tx.clone()).await;

                let broadcaster_meta_data = BroadcasterMetaData {
                    transmiter: tx.clone(),
                    broadcaster_peer: peer_connection,
                    track_channel_rx: local_track_chan_rx.recv().await.unwrap(),
                };

                let room = Room {
                    broadcaster: broadcaster_meta_data,
                    room_users: vec![],
                };
                println!("Broadcaster: {:?} created room {:?}", addr, room_name);

                let mut channels = channel_peer_map.lock().await;
                channels.insert(room_name, room);
            }
        }
    }
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
