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

    let token_owned = token.to_string();
    let stdin = tokio::io::stdin();
    let mut stdin_reader = BufReader::new(stdin).lines();
    let mut authenticated = false;

    loop {
        tokio::select! {
            // Read from WebSocket
            msg = read.next() => {
                match msg {
                    Some(Ok(msg)) => {
                        if msg.is_text() {
                            let text = msg.into_text().unwrap_or_default();

                            // Handle OpenClaw's connect.challenge auth flow
                            if !authenticated {
                                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                                    if parsed.get("event").and_then(|e| e.as_str()) == Some("connect.challenge") {
                                        // Respond with auth token
                                        let auth_response = serde_json::json!({
                                            "type": "auth",
                                            "token": token_owned
                                        });
                                        use tokio_tungstenite::tungstenite::protocol::Message;
                                        write.send(Message::Text(auth_response.to_string()))
                                            .await
                                            .context("Failed to send auth response")?;
                                        authenticated = true;
                                        output::success("Connected");
                                        println!();
                                        continue;
                                    }
                                }
                            }

                            // Parse and display chat messages nicely
                            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                                display_message(&parsed);
                            } else {
                                println!("{}", text);
                            }
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
                        if line.is_empty() {
                            continue;
                        }
                        // Send as a chat message
                        let msg = serde_json::json!({
                            "type": "message",
                            "content": line
                        });
                        use tokio_tungstenite::tungstenite::protocol::Message;
                        write.send(Message::Text(msg.to_string())).await
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

/// Display a parsed WebSocket message in a readable format
fn display_message(msg: &serde_json::Value) {
    let event = msg.get("event").and_then(|e| e.as_str()).unwrap_or("");
    let msg_type = msg.get("type").and_then(|t| t.as_str()).unwrap_or("");

    match (msg_type, event) {
        // Assistant text responses
        (_, "assistant.text") | (_, "message.text") => {
            if let Some(text) = msg
                .get("payload")
                .and_then(|p| p.get("text"))
                .and_then(|t| t.as_str())
            {
                print!("{}", text);
            }
        }
        // Assistant message complete
        (_, "assistant.done") | (_, "message.done") => {
            println!();
            println!();
        }
        // Tool use
        (_, "tool.start") => {
            if let Some(name) = msg
                .get("payload")
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
            {
                output::debug(&format!("Using tool: {}", name));
            }
        }
        // Error
        ("error", _) | (_, "error") => {
            if let Some(error) = msg
                .get("payload")
                .and_then(|p| p.get("message"))
                .and_then(|m| m.as_str())
            {
                output::error(error);
            } else if let Some(error) = msg.get("error").and_then(|e| e.as_str()) {
                output::error(error);
            }
        }
        // Connection confirmed
        (_, "connect.ok") | (_, "connected") => {
            // Already handled
        }
        // Everything else — show raw for debugging
        _ => {
            if !event.is_empty() || !msg_type.is_empty() {
                output::debug(&format!(
                    "Event: {} ({})",
                    if event.is_empty() { msg_type } else { event },
                    msg
                ));
            }
        }
    }
}
