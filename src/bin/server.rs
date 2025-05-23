use futures_util::{SinkExt, StreamExt};
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{channel, Sender};
use tokio_websockets::{
    Message,
    ServerBuilder,
    WebSocketStream
};

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>, // This should be the actual stream, not a tuple
    bcast_tx: Sender<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> { // Added Send + Sync for broader compatibility
    println!("New WebSocket connection established: {}", addr);
    let mut bcast_rx = bcast_tx.subscribe();

    loop {
        tokio::select! {
            Some(msg_result) = ws_stream.next() => {
                match msg_result {
                    Ok(msg) => {
                        if msg.is_text() {
                            // Corrected: Use as_text() and handle Option
                            if let Some(text_content) = msg.as_text() { // Line 24 (conceptual)
                                if !text_content.is_empty() {
                                    println!("Received from {}: {}", addr, text_content);
                                    let formatted_msg = format!("{}: {}", addr, text_content);
                                    if bcast_tx.send(formatted_msg).is_err() {
                                        eprintln!("Broadcast error (client: {})", addr);
                                    }
                                }
                            } else {
                                eprintln!("Message from {} is text but as_text() returned None.", addr);
                            }
                        } else if msg.is_close() {
                            println!("Client {} sent a close message.", addr);
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error receiving message from {}: {}", addr, e);
                        break;
                    }
                }
            }
            Ok(broadcast_msg) = bcast_rx.recv() => {
                if ws_stream.send(Message::text(broadcast_msg.clone())).await.is_err() {
                    eprintln!("Error sending broadcast message to {}: client disconnected?", addr);
                    break;
                }
            }
            else => {
                break;
            }
        }
    }
    println!("Connection with {} closed.", addr);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> { // Added Send + Sync
    let (bcast_tx, _) = channel(16);
    let listener = TcpListener::bind("127.0.0.1:2000").await?;
    println!("Server listening on ws://127.0.0.1:2000");

    loop {
        let (socket, addr) = listener.accept().await?;
        let bcast_tx_clone = bcast_tx.clone();

        tokio::spawn(async move {
            match ServerBuilder::new().accept(socket).await {
                Ok((_request, ws_stream_accepted)) => { // Use a different variable name here
                    // Now pass the correctly de-structured ws_stream_accepted
                    if let Err(e) = handle_connection(addr, ws_stream_accepted, bcast_tx_clone).await { // Line 74 (conceptual)
                        eprintln!("Error in handle_connection for {}: {}", addr, e);
                    }
                }
                Err(e) => {
                    eprintln!("WebSocket handshake error with {}: {}", addr, e);
                }
            }
        });
    }
}