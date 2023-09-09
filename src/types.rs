use crate::models::Room;
use futures_channel::mpsc::UnboundedSender;
use futures_util::stream::{SplitSink, SplitStream};
use webrtc::peer_connection::RTCPeerConnection;
use std::{collections::HashMap, sync::Arc};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

// use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
// use webrtc::util::sync::RwLock;
// pub type ChannelDataType = Arc<TrackLocalStaticRTP>;

// type Tx = mpsc::Sender<Message>;
//
// pub type Tx = tokio::sync::mpsc::Sender<Message>;
// tokio::sync::mpsc::

// pub type RoomMap = HashMap<RoomName, Room>;
// pub type BroadcastRoomMap = Arc<Mutex<RoomMap>>;
// pub type BroadcastRoomMap = Arc<Mutex<HashMap<String, models::Room>>>;

pub type Tx = UnboundedSender<Message>;
pub type PeerConn = Arc<RTCPeerConnection>;

// pub type Channel = HashMap<String, Room>;
pub type ClientsMap = Arc<Mutex<HashMap<String, Room>>>;

pub type WsSplit = (
    SplitSink<WebSocketStream<TcpStream>, Message>,
    SplitStream<WebSocketStream<TcpStream>>,
);
