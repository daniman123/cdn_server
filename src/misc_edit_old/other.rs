
// pub async fn _handle_viewer(desc: RTCSessionDescription, _room: Room, _addr: (SocketAddr, Tx)) {
//     let peer_connection = create_peer_connection().await;

//     peer_connection.set_remote_description(desc).await.unwrap();

//     // TODO -- get media tracks as arg
//     // let tracks = room.broadcast.capability.channels;

//     // peer_connection.add_track(track);

//     let answer = peer_connection.create_answer(None).await.unwrap();

//     peer_connection.set_local_description(answer).await.unwrap();

//     let local_desc = peer_connection.local_description().await.unwrap();

//     let payload = json!(local_desc);

//     println!("WE ARE IN THE ");

//     let _answer = Message::Text(payload.to_string());

//     // send_payload(channel_peer_map, addr, answer).await;
// }


// pub async fn _handle_broadcaster_role(
//     broadcast_payload: BroadCastPayload,
//     channel_peer_map: &BroadcastRoomMap,
//     tx: &Tx,
//     addr: SocketAddr,
// ) {
//     let event_type = broadcast_payload.eventType;
//     let room_name = broadcast_payload.roomName;
//     println!("{:?}", room_name);
//     let channels = channel_peer_map.as_ref();

//     if event_type == "START_BROADCAST" {
//         if let Some(desc) = broadcast_payload.localDesc {
//             let mut media_engine = MediaEngine::default();

//             media_engine.register_default_codecs().unwrap();

//             let registry = Registry::new();
//             let registry = register_default_interceptors(registry, &mut media_engine).unwrap();
//             let api = APIBuilder::new()
//                 .with_media_engine(media_engine)
//                 .with_interceptor_registry(registry)
//                 .build();

//             let config = rtc_config();

//             let peer_connection = Arc::new(api.new_peer_connection(config).await.unwrap());

//             // println!("{:?}", desc);
//             let room = Room::create_room((addr, tx.clone()));
//             channels
//                 .lock()
//                 .unwrap()
//                 .insert(room_name.clone(), room.clone());
//             // channels.insert(room_name.clone(), room.clone());

//             tokio::spawn(handle_broadcaster(peer_connection.clone(), desc, room));
//         }
//     }

// if event_type == "TERMINATE_BROADCAST" {
//     channels.remove(&room_name.clone());
// }
// }

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