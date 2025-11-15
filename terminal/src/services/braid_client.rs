//! # Braid Client Service
//!
//! Client for Braid HTTP protocol - handles SSE subscriptions and PUT requests for messaging.

use shared::dto::messaging::Message;
use tokio::sync::mpsc;
use futures_util::StreamExt;

/// Braid client for a single conversation
pub struct BraidClient {
    conversation_id: String,
    token: String,
    base_url: String,
    last_version: Option<String>,
}

impl BraidClient {
    pub fn new(conversation_id: String, token: String) -> Self {
        Self {
            conversation_id,
            token,
            base_url: "http://127.0.0.1:3001".to_string(),
            last_version: None,
        }
    }

    /// Subscribe to conversation updates via SSE
    /// Returns a receiver channel that receives message updates
    pub async fn subscribe(&mut self) -> Result<mpsc::Receiver<(Vec<Message>, String)>, String> {
        let (tx, rx) = mpsc::channel(100);
        let conversation_id = self.conversation_id.clone();
        let token = self.token.clone();
        let base_url = self.base_url.clone();
        let mut last_version = self.last_version.clone();
        
        tokio::spawn(async move {
            let url = format!("{}/api/chat/{}", base_url, conversation_id);
            
            let client = reqwest::Client::new();
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert("subscribe", reqwest::header::HeaderValue::from_static(""));
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))
                    .unwrap_or_else(|_| reqwest::header::HeaderValue::from_static("")),
            );
            
            if let Some(parents) = &last_version {
                if let Ok(header_value) = reqwest::header::HeaderValue::from_str(parents) {
                    headers.insert("parents", header_value);
                }
            }
            
            let response = match client.get(&url).headers(headers).send().await {
                Ok(resp) => resp,
                Err(e) => {
                    eprintln!("Failed to subscribe to conversation: {}", e);
                    return;
                }
            };
            
            if !response.status().is_success() {
                eprintln!("Subscription failed with status: {}", response.status());
                return;
            }
            
            let mut stream = response.bytes_stream();
            
            let mut buffer = String::new();
            
            while let Some(item) = stream.next().await {
                match item {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        buffer.push_str(&text);
                        
                        // Parse SSE events (format: "data: {json}\n\n")
                        for line in buffer.lines() {
                            if line.starts_with("data: ") {
                                let json_str = &line[6..]; // Skip "data: "
                                
                                match serde_json::from_str::<serde_json::Value>(json_str) {
                                    Ok(event_data) => {
                                        if let (Some(version), Some(messages_array)) = (
                                            event_data.get("version").and_then(|v| v.as_str()),
                                            event_data.get("messages").and_then(|m| m.as_array()),
                                        ) {
                                            let version = version.to_string();
                                            
                                            let messages: Vec<Message> = messages_array
                                                .iter()
                                                .filter_map(|m| {
                                                    serde_json::from_value(m.clone()).ok()
                                                })
                                                .collect();
                                            
                                            if let Err(_) = tx.send((messages, version)).await {
                                                // Receiver dropped, stop subscription
                                                return;
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to parse SSE event: {}", e);
                                    }
                                }
                            }
                        }
                        
                        // Clear processed lines from buffer (keep incomplete line if any)
                        if let Some(last_newline) = buffer.rfind('\n') {
                            buffer = buffer[last_newline + 1..].to_string();
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading SSE stream: {}", e);
                        break;
                    }
                }
            }
        });
        
        Ok(rx)
    }

    /// Send a message via Braid PUT
    pub async fn send_message(&mut self, message: Message) -> Result<String, String> {
        let url = format!("{}/api/chat/{}", self.base_url, self.conversation_id);
        
        let client = reqwest::Client::new();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", self.token))
                .map_err(|e| format!("Failed to create auth header: {}", e))?,
        );
        
        if let Some(parents) = &self.last_version {
            headers.insert(
                "parents",
                reqwest::header::HeaderValue::from_str(parents)
                    .map_err(|e| format!("Failed to create parents header: {}", e))?,
            );
        }
        
        let body = serde_json::to_string(&message)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;
        
        let response = client
            .put(&url)
            .headers(headers)
            .body(body)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
        
        if response.status().is_success() {
            // Get version from response header
            if let Some(version_header) = response.headers().get("version") {
                if let Ok(version) = version_header.to_str() {
                    let version = version.to_string();
                    self.last_version = Some(version.clone());
                    return Ok(version);
                }
            }
            Ok(String::new())
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("API error: {}", error_text))
        }
    }

    /// Send typing indicator
    pub async fn send_typing(&self, is_typing: bool) -> Result<(), String> {
        let url = format!("{}/api/chat/{}/typing", self.base_url, self.conversation_id);
        
        let client = reqwest::Client::new();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", self.token))
                .map_err(|e| format!("Failed to create auth header: {}", e))?,
        );
        
        let body = serde_json::json!({
            "is_typing": is_typing
        });
        
        let response = client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("API error: {}", error_text))
        }
    }
}

