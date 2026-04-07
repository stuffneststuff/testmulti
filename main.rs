use std::{collections::HashMap, sync::{Arc, Mutex}};
use futures_util::{StreamExt, SinkExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;

type PeerMap = Arc<Mutex<HashMap<std::net::SocketAddr, futures_channel::mpsc::UnboundedSender<tokio_tungstenite::tungstenite::Message>>>>;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    let peers = PeerMap::new(Mutex::new(HashMap::new()));

    println!("Server listening on: {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(peers.clone(), stream, addr));
    }
}

async fn handle_connection(peers: PeerMap, stream: TcpStream, addr: std::net::SocketAddr) {
    let ws_stream = accept_async(stream).await.expect("Error during websocket handshake");
    let (tx, mut rx) = futures_channel::mpsc::unbounded();
    peers.lock().unwrap().insert(addr, tx);

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    let broadcast_incoming = async {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            let peers = peers.lock().unwrap();
            for (&peer_addr, rec) in peers.iter() {
                if peer_addr != addr {
                    let _ = rec.unbounded_send(msg.clone());
                }
            }
        }
    };

    let receive_from_others = async {
        while let Some(msg) = rx.next().await {
            let _ = ws_sender.send(msg).await;
        }
    };

    tokio::select! {
        _ = broadcast_incoming => {},
        _ = receive_from_others => {},
    }

    peers.lock().unwrap().remove(&addr);
    println!("{} disconnected", addr);
}
