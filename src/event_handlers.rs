use anyhow::Result;
use tokio::sync::mpsc::Receiver;
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

use webrtc::{
    ice_transport::ice_candidate::RTCIceCandidateInit,
    peer_connection::RTCPeerConnection,
    rtcp::payload_feedbacks::picture_loss_indication::PictureLossIndication,
    track::track_local::{track_local_static_rtp::TrackLocalStaticRTP, TrackLocalWriter},
    Error,
};

pub async fn handle_track_event(
    peer_connection: Arc<RTCPeerConnection>,
    pc: std::sync::Weak<webrtc::peer_connection::RTCPeerConnection>,
    local_track_chan_tx: tokio::sync::mpsc::Sender<Arc<TrackLocalStaticRTP>>,
) {
    let local_track_chan_tx = Arc::new(local_track_chan_tx);

    peer_connection.on_track(Box::new(move |track, _, _| {
        // println!("kodak {:?}", track.codec());
        // Send a PLI on an interval so that the publisher is pushing a keyframe every rtcpPLIInterval
        // This is a temporary fix until we implement incoming RTCP events, then we would push a PLI only when a viewer requests it
        let media_ssrc = track.ssrc();

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
}

pub async fn handle_candidate_event(
    peer_connection: Arc<RTCPeerConnection>,
) -> Receiver<RTCIceCandidateInit> {
    let (gathering_complete_tx, gathering_complete_rx) = tokio::sync::mpsc::channel(1);

    let done = Arc::new(Mutex::new(Some(gathering_complete_tx)));
    let done2 = Arc::clone(&done);

    peer_connection.on_ice_candidate(Box::new(move |candidate| {
        let done3 = Arc::clone(&done2);
        // Process the received ICE candidate
        // Add the ICE candidate to the RTCPeerConnection
        Box::pin(async move {
            if let Some(cand) = candidate {
                let dater = cand.to_json().unwrap();

                let ice = RTCIceCandidateInit {
                    candidate: dater.candidate,
                    username_fragment: dater.username_fragment,
                    sdp_mid: dater.sdp_mid,
                    sdp_mline_index: dater.sdp_mline_index,
                };
                // println!("jeezy {:?}", ice);
                done3.lock().await.clone().unwrap().send(ice).await.unwrap();
            }
        })
    }));

    gathering_complete_rx
}
