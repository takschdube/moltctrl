use anyhow::{bail, Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

use crate::output;

/// Start an interactive WebSocket chat session
pub async fn start_chat(port: u16, token: &str) -> Result<()> {
    let url = format!("ws://127.0.0.1:{}/ws", port);

    output::info(&format!("Connecting to {}...", url));
    output::info("Press Ctrl+C to disconnect");
    println!();

    let mut request = url
        .into_client_request()
        .context("Failed to create WebSocket request")?;
    request.headers_mut().insert(
        "Authorization",
        format!("Bearer {}", token)
            .parse()
            .context("Invalid token header")?,
    );

    let (ws_stream, _) = tokio_tungstenite::connect_async(request)
        .await
        .context("Failed to connect to WebSocket")?;

    let (mut write, mut read) = ws_stream.split();

    let stdin = tokio::io::stdin();
    let mut stdin_reader = BufReader::new(stdin).lines();

    loop {
        tokio::select! {
            // Read from WebSocket
            msg = read.next() => {
                match msg {
                    Some(Ok(msg)) => {
                        if msg.is_text() {
                            println!("{}", msg.into_text().unwrap_or_default());
                        } else if msg.is_close() {
                            output::info("Connection closed by server");
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        bail!("WebSocket error: {}", e);
                    }
                    None => {
                        output::info("Connection closed");
                        break;
                    }
                }
            }
            // Read from stdin
            line = stdin_reader.next_line() => {
                match line {
                    Ok(Some(line)) => {
                        use tokio_tungstenite::tungstenite::protocol::Message;
                        write.send(Message::Text(line)).await
                            .context("Failed to send message")?;
                    }
                    Ok(None) => {
                        // EOF on stdin
                        break;
                    }
                    Err(e) => {
                        bail!("stdin error: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}
