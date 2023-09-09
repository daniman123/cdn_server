use std::{net::SocketAddr, sync::Arc};

// use futures_channel::mpsc::Sender;
use serde_json::json;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::tungstenite::Message;
use webrtc::{
    peer_connection::peer_connection_state::RTCPeerConnectionState,
    rtp_transceiver::rtp_codec::RTPCodecType,
    track::track_local::track_local_static_rtp::TrackLocalStaticRTP,
};

use crate::{
    event_handlers::handle_track_event,
    models::{BroadCastPayload, Room},
    types::{BroadcastRoomMap, RoomMap, Tx},
    utils::create_peer_connection,
};

pub async fn handle_broadcaster_role(
    local_track_chan_tx: Sender<Arc<TrackLocalStaticRTP>>,
    broadcast_payload: BroadCastPayload,
    // channel_peer_map: &BroadcastRoomMap,
    tx: &Tx,
    addr: SocketAddr,
) -> Result<Room, ()> {
    let event_type = broadcast_payload.eventType;
    let room_name = broadcast_payload.roomName;


    println!("{:?}", room_name);

    // if event_type == "START_BROADCAST" {
        if let Some(desc) = broadcast_payload.localDesc {
            let peer_connection = create_peer_connection().await;

            peer_connection
                .add_transceiver_from_kind(RTPCodecType::Video, None)
                .await
                .unwrap();

            // peer_connection
            //     .add_transceiver_from_kind(RTPCodecType::Audio, None)
            //     .await
            //     .unwrap();

            // let (local_track_chan_tx, mut local_track_chan_rx) =
            //     tokio::sync::mpsc::channel::<Arc<TrackLocalStaticRTP>>(1);

            let local_track_chan_tx = Arc::new(local_track_chan_tx);
            let pc = Arc::downgrade(&peer_connection);

            let room = Room::create_room((addr, tx.clone()));
            // let room = Arc::new(Room::create_room((addr, tx.clone())));

            handle_track_event(peer_connection.clone(), pc, local_track_chan_tx);

            // Set the handler for Peer connection state
            // This will notify you when the peer has connected/disconnected
            peer_connection.on_peer_connection_state_change(Box::new(
                move |s: RTCPeerConnectionState| {
                    println!("Peer Connection State has changed: {s}");
                    Box::pin(async {})
                },
            ));

            // Set the remote SessionDescription
            peer_connection.set_remote_description(desc).await.unwrap();

            // Create an answer
            let local_desc = peer_connection.create_answer(None).await.unwrap();

            let mut gather_complete = peer_connection.gathering_complete_promise().await;

            // Sets the LocalDescription, and starts our UDP listeners
            peer_connection
                .set_local_description(local_desc.clone())
                .await
                .unwrap();

            // Block until ICE Gathering is complete, disabling trickle ICE
            // we do this because we only can exchange one signaling message
            // in a production application you should exchange ICE Candidates via OnICECandidate
            let _ = gather_complete.recv().await;

            let localo_desco = peer_connection.local_description().await.unwrap();

            let payload = json!(localo_desco);
            let answer = Message::Text(payload.to_string());

            if let Err(err) = tx.unbounded_send(answer.clone()) {
                eprintln!("Failed to send message to recipient: {} {:?}", err, tx);
            }

            // println!("{:?}", channel_peer_map);

            Ok(room)
        } else {
            Err(())
        }
    // } else {
        // Err(())
    // }
}
