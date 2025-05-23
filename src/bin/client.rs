// src/bin/client.rs
use futures_util::{SinkExt, StreamExt};
use std::error::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use http::Uri; // Correct import for Uri
use tokio_websockets::{ClientBuilder, Message};
// use tokio_websockets::core::CloseCode; // If specific close codes are needed

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> { // Added Send + Sync
    let server_addr = "ws://127.0.0.1:8080";  // Ensure this matches the server
    let uri = server_addr.parse::<Uri>()?;

    // The connect() method for ClientBuilder usually returns a tuple:
    // (WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>, http::Response<Option<Vec<u8>>>)
    // We are interested in the first part, the stream.
    let (ws_stream, _response) = ClientBuilder::from_uri(uri)
        .connect()
        .await
        .map_err(|e| format!("Can't connect to {}: {}", server_addr, e))?;

    println!("Connected to server: {}", server_addr);
    println!("Type your message and press Enter. Type 'exit' or 'quit' to close.");

    let (mut ws_sender, mut ws_reader) = ws_stream.split();
    let mut stdin_reader = BufReader::new(tokio::io::stdin()).lines();

    // This is the client's main loop, not a call to a server-style handle_connection
    loop {
        tokio::select! {
            Ok(Some(line)) = stdin_reader.next_line() => {
                if line.trim().eq_ignore_ascii_case("exit") || line.trim().eq_ignore_ascii_case("quit") {
                    println!("Exiting...");
                    // Provide an empty string as the reason if none is specific
                    if ws_sender.send(Message::close(None, "")).await.is_err() {
                        eprintln!("Error sending close message.");
                    }
                    break;
                }
                if let Err(e) = ws_sender.send(Message::text(line)).await {
                    eprintln!("Error sending message: {}", e);
                    break;
                }
            }
            Some(msg_result) = ws_reader.next() => {
                match msg_result {
                    Ok(msg) => {
                        if msg.is_text() {
                            if let Some(text_content) = msg.as_text() { // Use as_text()
                                println!("Received: {}", text_content);
                            } else {
                                eprintln!("Received text message but as_text() returned None.");
                            }
                        } else if msg.is_close() {
                            println!("Server closed connection.");
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error receiving message: {}", e);
                        break;
                    }
                }
            }
            else => break,
        }
    }
    Ok(())
}