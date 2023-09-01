use crate::{
    helpers::accept_and_split,
    models::{BroadCastPayload, Room},
    types::{BroadcastRoomMap, Tx},
    utils::{handle_broadcaster, rtc_config},
};
use anyhow::Result;
use futures_channel::mpsc::unbounded;
use futures_util::{future, pin_mut, StreamExt, TryStreamExt};
use serde_json::{json, Value};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    net::TcpStream,
    signal,
    sync::mpsc::{unbounded_channel, UnboundedReceiver},
};
use tokio_tungstenite::tungstenite::Message;
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors, media_engine::MediaEngine, APIBuilder,
    },
    interceptor::registry::Registry,
    peer_connection::peer_connection_state::RTCPeerConnectionState,
    rtcp::payload_feedbacks::picture_loss_indication::PictureLossIndication,
    rtp_transceiver::rtp_codec::RTPCodecType,
    track::track_local::{track_local_static_rtp::TrackLocalStaticRTP, TrackLocalWriter},
    Error,
};

pub async fn handle_connection(
    channel_peer_map: BroadcastRoomMap,
    raw_stream: TcpStream,
    addr: SocketAddr,
) {
    println!("Incoming TCP connection from: {}", addr);
    let (tx, rx) = unbounded();
    let (data_tx, data_rx) = unbounded_channel(); // Channel for data streaming task

    let (outgoing, incoming) = accept_and_split(raw_stream).await;
    println!("WebSocket connection established: {}", addr);

    let broadcast_incoming = incoming.try_for_each(|msg| {
        if let Ok(json) = msg.to_text().map_err(|_| ()) {
            if let Ok(payload) = serde_json::from_str::<Value>(&json) {
                // Pass information to the data streaming task

                data_tx.send(payload).ok();
                // payload_extractor(payload,  &channel_peer_map, &tx, addr);
            }
        }

        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);
    // Spawn a task to continuously stream data to the client
    tokio::spawn(stream_data_to_client(
        data_rx,
        channel_peer_map.clone(),
        tx.clone(),
        addr,
    ));
    // println!("{:?}", channel_peer_map.lock().unwrap());

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;
}

async fn stream_data_to_client(
    mut data_rx: UnboundedReceiver<Value>,
    channel_peer_map: BroadcastRoomMap,
    tx: Tx,
    addr: SocketAddr,
) {
    loop {
        // Wait for information from messages
        if let Some(info) = data_rx.recv().await {
            println!("stream_data_to_client {:?}", info);
            payload_extractor(info, &channel_peer_map, &tx, addr).await;
        }
    }
}

pub async fn payload_extractor(
    payload: Value,
    channel_peer_map: &BroadcastRoomMap,
    tx: &Tx,
    addr: SocketAddr,
) {
    if let Some(Value::Object(obj)) = payload.get("BROADCASTER") {
        if let Ok(payload) = serde_json::from_value::<BroadCastPayload>(Value::Object(obj.clone()))
        {
            // data_tx.send(payload.clone()).ok();
            handle_broadcaster_role(payload, &channel_peer_map.clone(), &tx, addr).await;
        }
    }

    // if let Some(Value::Object(obj)) = payload.get("VIEWER") {
    //     if let Ok(payload) = serde_json::from_value::<ViewerPayload>(Value::Object(obj.clone())) {
    //         handle_viewer_role(payload, channel_peer_map, tx, addr);
    //     }
    // }
}

pub async fn handle_broadcaster_role(
    broadcast_payload: BroadCastPayload,
    channel_peer_map: &BroadcastRoomMap,
    tx: &Tx,
    addr: SocketAddr,
) {
    let event_type = broadcast_payload.eventType;
    let room_name = broadcast_payload.roomName;
    println!("{:?}", room_name);
    let channels = channel_peer_map.as_ref();

    if event_type == "START_BROADCAST" {
        if let Some(desc) = broadcast_payload.localDesc {
            let mut media_engine = MediaEngine::default();

            media_engine.register_default_codecs().unwrap();

            let registry = Registry::new();
            let registry = register_default_interceptors(registry, &mut media_engine).unwrap();
            let api = APIBuilder::new()
                .with_media_engine(media_engine)
                .with_interceptor_registry(registry)
                .build();

            let config = rtc_config();

            let peer_connection = Arc::new(api.new_peer_connection(config).await.unwrap());

            peer_connection
                .add_transceiver_from_kind(RTPCodecType::Video, None)
                .await
                .unwrap();

            let (local_track_chan_tx, mut local_track_chan_rx) =
                tokio::sync::mpsc::channel::<Arc<TrackLocalStaticRTP>>(1);

            let local_track_chan_tx = Arc::new(local_track_chan_tx);
            let pc = Arc::downgrade(&peer_connection);

            peer_connection.on_track(Box::new(move |track, _, _| {
                println!("kodak {:?}", track.codec());
                // Send a PLI on an interval so that the publisher is pushing a keyframe every rtcpPLIInterval
                // This is a temporary fix until we implement incoming RTCP events, then we would push a PLI only when a viewer requests it
                let media_ssrc = track.ssrc();
                println!("{:?}", media_ssrc);
                let pc2 = pc.clone();
                tokio::spawn(async move {
                    let mut result = Result::<usize>::Ok(0);
                    while result.is_ok() {
                        let timeout = tokio::time::sleep(Duration::from_secs(3));
                        tokio::pin!(timeout);

                        tokio::select! {
                            _ = timeout.as_mut() =>{
                                if let Some(pc) = pc2.upgrade(){
                                    result = pc.write_rtcp(&[Box::new(PictureLossIndication{
                                        sender_ssrc: 0,
                                        media_ssrc,
                                    })]).await.map_err(Into::into);
                                }else{
                                    break;
                                }
                            }
                        };
                    }
                });

                let local_track_chan_tx2 = Arc::clone(&local_track_chan_tx);
                tokio::spawn(async move {
                    // Create Track that we send video back to browser on
                    let local_track = Arc::new(TrackLocalStaticRTP::new(
                        track.codec().capability,
                        "video".to_owned(),
                        "webrtc-rs".to_owned(),
                    ));
                    let _ = local_track_chan_tx2.send(Arc::clone(&local_track)).await;

                    // Read RTP packets being sent to webrtc-rs
                    while let Ok((rtp, _)) = track.read_rtp().await {
                        if let Err(err) = local_track.write_rtp(&rtp).await {
                            if Error::ErrClosedPipe != err {
                                print!("output track write_rtp got error: {err} and break");
                                break;
                            } else {
                                print!("output track write_rtp got error: {err}");
                            }
                        }
                    }
                });

                Box::pin(async {})
            }));

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
            peer_connection.set_local_description(local_desc.clone()).await.unwrap();

            // Block until ICE Gathering is complete, disabling trickle ICE
            // we do this because we only can exchange one signaling message
            // in a production application you should exchange ICE Candidates via OnICECandidate
            let _ = gather_complete.recv().await;

            // peer_connection
            //     .set_local_description(local_desc.clone())
            //     .await
            //     .unwrap();

            let localo_desco = peer_connection.local_description().await.unwrap();


            let payload = json!(localo_desco);
            let answer = Message::Text(payload.to_string());

            if let Err(err) = tx.unbounded_send(answer.clone()) {
                eprintln!("Failed to send message to recipient: {} {:?}", err, tx);
            }
        }
    }
}

pub async fn _handle_broadcaster_role(
    broadcast_payload: BroadCastPayload,
    channel_peer_map: &BroadcastRoomMap,
    tx: &Tx,
    addr: SocketAddr,
) {
    let event_type = broadcast_payload.eventType;
    let room_name = broadcast_payload.roomName;
    println!("{:?}", room_name);
    let channels = channel_peer_map.as_ref();

    if event_type == "START_BROADCAST" {
        if let Some(desc) = broadcast_payload.localDesc {
            let mut media_engine = MediaEngine::default();

            media_engine.register_default_codecs().unwrap();

            let registry = Registry::new();
            let registry = register_default_interceptors(registry, &mut media_engine).unwrap();
            let api = APIBuilder::new()
                .with_media_engine(media_engine)
                .with_interceptor_registry(registry)
                .build();

            let config = rtc_config();

            let peer_connection = Arc::new(api.new_peer_connection(config).await.unwrap());

            // println!("{:?}", desc);
            let room = Room::create_room((addr, tx.clone()));
            channels
                .lock()
                .unwrap()
                .insert(room_name.clone(), room.clone());
            // channels.insert(room_name.clone(), room.clone());

            tokio::spawn(handle_broadcaster(peer_connection.clone(), desc, room));
        }
    }

    // if event_type == "TERMINATE_BROADCAST" {
    //     channels.remove(&room_name.clone());
    // }
}

// pub fn handle_viewer_role(
//     viewer_payload: ViewerPayload,
//     channel_peer_map: &BroadcastRoomMap,
//     tx: &Tx,
//     addr: SocketAddr,
// ) {
//     let event_type = viewer_payload.eventType;
//     let room_name = viewer_payload.roomName;
//     println!("{:?}", room_name);
//     let mut channels = channel_peer_map.lock().unwrap();

//     if event_type == "ENTER_ROOM" {
//         if let Some(desc) = viewer_payload.localDesc {
//             channels
//                 .get(&room_name)
//                 .unwrap()
//                 .room_users
//                 .push((addr, tx.clone()));

//             // println!("{:?}", desc);
//             // let room = Room::create_room((addr, tx.clone()));
//             // channels.insert(room_name.clone(), room.clone());

//             tokio::spawn(handle_viewer(desc, room, (addr, tx.clone())));
//         }
//     }

//     if event_type == "LEAVE_ROOM" {
//         channels.remove(&room_name.clone());
//     }
// }
