use futures_channel::mpsc::UnboundedSender;
use futures_util::stream::{SplitSink, SplitStream};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
// use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecParameters;

pub type Tx = UnboundedSender<Message>;

pub type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;
pub type WsSplit = (
    SplitSink<WebSocketStream<TcpStream>, Message>,
    SplitStream<WebSocketStream<TcpStream>>,
);

// pub type RoomName = String;
// pub type RoomMembers = Vec<SocketAddr>;
// pub type RoomBroadcast = RTCRtpCodecParameters;

// pub type RoomBroadcastContainer = HashMap<RoomBroadcast, RoomMembers>;

// pub type BroadcastRoomMap = Arc<Mutex<HashMap<RoomName, RoomBroadcastContainer>>>;
