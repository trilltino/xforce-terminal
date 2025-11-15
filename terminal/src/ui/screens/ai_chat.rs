//! # AI Chat Screen
//!
//! Chat interface for talking to the AI assistant. Uses the Braid messaging protocol
//! to communicate with the backend AI bot.

use egui;
use crate::app::{AppState, AppLike};
use crate::ui::theme::Theme;
use std::sync::Arc;
use parking_lot::RwLock;

/// Render AI chat screen
pub fn render(ui: &mut egui::Ui, state: &AppState, app: &mut impl AppLike) {
    let theme = Theme::default();
    
    // Clone app.state at the beginning to avoid borrow conflicts in closures
    let app_state = app.state().clone();
    
    // Request immediate repaint if AI is typing (for smooth animation)
    // Get context here but don't hold the borrow
    if state.ai_chat.ai_typing {
        let ctx = ui.ctx();
        ctx.request_repaint();
    }
    
    // Initialize conversation if not already done
    if state.ai_chat.conversation_id.is_none() {
        if let Some(current_user) = &state.current_user {
            let mut state_write = app_state.write();
            // Use conversation ID format: "0:{user_id}" where 0 is the AI bot user ID
            // The format requires smaller ID first, so 0 comes before any positive user ID
            let conversation_id = format!("0:{}", current_user.id);
            state_write.ai_chat.conversation_id = Some(conversation_id.clone());
            
            // Start SSE subscription for AI conversation
            let token_opt = state_write.auth_token.clone();
            let conversation_id_for_sub = conversation_id.clone();
            let state_for_task = app_state.clone();
            
            drop(state_write);
            
            if let Some(token) = token_opt {
                let conversation_id_clone = conversation_id_for_sub.clone();
                tokio::spawn(async move {
                    // Subscribe to conversation updates
                    let mut braid_client = crate::services::braid_client::BraidClient::new(
                        conversation_id_for_sub,
                        token,
                    );
                    
                    match braid_client.subscribe().await {
                        Ok(mut rx) => {
                            {
                                let mut state = state_for_task.write();
                                state.ai_chat.subscribed = true;
                            }
                            tracing::info!(
                                conversation_id = %conversation_id_clone,
                                "SSE subscription established for AI chat - waiting for messages"
                            );
                            
                            while let Some((messages, version)) = rx.recv().await {
                                let message_count = messages.len();
                                let ai_message_count = messages.iter()
                                    .filter(|msg| {
                                        let author_lower = msg.author.to_lowercase();
                                        author_lower.contains("deepseek") 
                                            || author_lower.contains("ai")
                                            || author_lower.contains("assistant")
                                            || msg.author == "DeepSeek AI"
                                    })
                                    .count();
                                
                                tracing::debug!(
                                    conversation_id = %conversation_id_clone,
                                    version = %version,
                                    total_messages = message_count,
                                    ai_messages = ai_message_count,
                                    "SSE: Received message update - updating UI state"
                                );
                                
                                let mut state = state_for_task.write();
                                let old_count = state.ai_chat.messages.len();
                                state.ai_chat.messages = messages.clone();
                                state.ai_chat.ai_typing = false;
                                drop(state);
                                
                                if message_count != old_count {
                                    tracing::info!(
                                        conversation_id = %conversation_id_clone,
                                        old_count = old_count,
                                        new_count = message_count,
                                        ai_messages = ai_message_count,
                                        "SSE: Message count changed - UI should update on next frame"
                                    );
                                }
                                // UI will update on next frame
                            }
                            
                            tracing::warn!(
                                conversation_id = %conversation_id_clone,
                                "SSE subscription ended - no more messages will be received"
                            );
                        }
                        Err(e) => {
                            tracing::error!(
                                conversation_id = %conversation_id_clone,
                                error = %e,
                                "Failed to subscribe to AI conversation via SSE"
                            );
                            eprintln!("Failed to subscribe to AI conversation: {}", e);
                            let mut state = state_for_task.write();
                            state.ai_chat.subscribed = false;
                        }
                    }
                });
            }
        }
    }
    
    use crate::ui::widgets::layouts;
    
    layouts::render_panel(ui, None, |ui| {
            // Header with dynamic WebSocket status
            ui.horizontal(|ui| {
                ui.heading("🤖 AI Assistant");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Dynamic status based on WebSocket connection state
                    let ws_status = &state.websocket_status;
                    let status_text = match ws_status.state {
                        crate::app::WebSocketState::Connected => {
                            if ws_status.messages_received > 0 {
                                format!("● Connected ({} msgs)", ws_status.messages_received)
                            } else {
                                "● Connected".to_string()
                            }
                        },
                        crate::app::WebSocketState::Connecting => "○ Connecting...".to_string(),
                        crate::app::WebSocketState::Reconnecting => {
                            format!("○ Reconnecting (attempt {})...", ws_status.connection_attempts)
                        },
                        crate::app::WebSocketState::Disconnected => "○ Disconnected".to_string(),
                        crate::app::WebSocketState::Disabled => "○ Disabled".to_string(),
                    };
                    
                    let status_color = match ws_status.state {
                        crate::app::WebSocketState::Connected => theme.success,
                        crate::app::WebSocketState::Connecting | crate::app::WebSocketState::Reconnecting => theme.warning,
                        _ => theme.dim,
                    };
                    
                    ui.colored_label(status_color, status_text);
                    
                    // Also show SSE subscription status for AI chat
                    if state.ai_chat.subscribed {
                        ui.colored_label(theme.success, " | SSE: Active");
                    } else {
                        ui.colored_label(theme.dim, " | SSE: Inactive");
                    }
                });
            });
        
        ui.separator();
        
        // Two-box layout: AI Response (top) and User Input (bottom)
        ui.vertical(|ui| {
            // Debug logging for UI rendering
            let message_count = state.ai_chat.messages.len();
            let ai_message_count = state.ai_chat.messages.iter()
                .filter(|msg| {
                    let author_lower = msg.author.to_lowercase();
                    author_lower.contains("deepseek") 
                        || author_lower.contains("ai")
                        || author_lower.contains("assistant")
                        || msg.author == "DeepSeek AI"
                })
                .count();
            
            tracing::debug!(
                conversation_id = ?state.ai_chat.conversation_id,
                subscribed = state.ai_chat.subscribed,
                total_messages = message_count,
                ai_messages = ai_message_count,
                ai_typing = state.ai_chat.ai_typing,
                "UI RENDER: Rendering AI chat interface"
            );
            
            // AI Response Box (top, read-only)
            ui.label("AI Response:");
            let ai_response_text = {
                // Get the latest AI message
                let latest_ai_message_text = state.ai_chat.messages.iter()
                    .rev()
                    .find(|msg| {
                        let author_lower = msg.author.to_lowercase();
                        author_lower.contains("deepseek") 
                            || author_lower.contains("ai")
                            || author_lower.contains("assistant")
                            || msg.author == "DeepSeek AI"
                    })
                    .map(|msg| {
                        tracing::debug!(
                            author = %msg.author,
                            message_length = msg.text.len(),
                            "UI RENDER: Found latest AI message"
                        );
                        msg.text.clone()
                    })
                    .unwrap_or_else(|| {
                        if state.ai_chat.messages.is_empty() {
                            tracing::debug!("UI RENDER: No messages yet - showing welcome message");
                            "Welcome! Ask me anything about Solana, DeFi, trading, or general questions. I'm here to help!".to_string()
                        } else {
                            tracing::warn!(
                                total_messages = message_count,
                                "UI RENDER: Messages exist but no AI message found - showing waiting message"
                            );
                            "Waiting for AI response...".to_string()
                        }
                    });
                
                // Animated typing indicator
                if state.ai_chat.ai_typing {
                    // Get current time for animation (get context inside closure)
                    let ctx = ui.ctx();
                    let current_time = ctx.input(|i| i.time);
                    
                    // Animate typing dots: ".", "..", "...", "." (cycles every 0.5 seconds)
                    let animation_phase = ((current_time * 2.0) as usize) % 4; // 0, 1, 2, 3
                    let typing_dots = match animation_phase {
                        0 => ".",
                        1 => "..",
                        2 => "...",
                        _ => "   ", // Empty for a brief moment before cycling
                    };
                    
                    format!("{latest_ai_message_text}\n\n🤖 AI is thinking{}", typing_dots)
                } else {
                    latest_ai_message_text
                }
            };
            
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(300.0)
                .show(ui, |ui| {
                    let mut response_text = ai_response_text.clone();
                    let text_edit = egui::TextEdit::multiline(&mut response_text)
                        .desired_width(f32::INFINITY)
                        .interactive(false) // Read-only
                        .hint_text("AI responses will appear here...");
                    ui.add(text_edit);
                });
            
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);
            
            // User Input Box (bottom, editable)
            ui.label("Your Message:");
            
            // Use try_write to avoid blocking
            let (message_text, should_send) = {
                match app_state.try_write() {
                    Some(mut state_write) => {
                        let text_edit = egui::TextEdit::multiline(&mut state_write.ai_chat.message_input)
                            .desired_width(f32::INFINITY)
                            .hint_text("Type your message to the AI...");
                        
                        let response = ui.add(text_edit);
                        let message_text = state_write.ai_chat.message_input.clone();
                        drop(state_write);
                        
                        let should_send = (response.lost_focus() && response.ctx.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift))
                            || ui.button("Send").clicked();
                        
                        (message_text, should_send)
                    }
                    None => {
                        // Lock is held, show a non-interactive text area
                        let mut temp_text = "Waiting...".to_string();
                        let text_edit = egui::TextEdit::multiline(&mut temp_text)
                            .desired_width(f32::INFINITY)
                            .interactive(false)
                            .hint_text("Please wait...");
                        ui.add(text_edit);
                        (String::new(), false)
                    }
                }
            };
            
            if should_send && !message_text.trim().is_empty() {
                send_message(app_state.clone(), message_text.clone());
            }
            
            // Help text
            ui.add_space(5.0);
            ui.colored_label(theme.dim, "Press Enter to send, Shift+Enter for new line");
        });
    });
}

/// Send a message to the AI
fn send_message(app_state: Arc<RwLock<AppState>>, text: String) {
    // Use try_read to avoid blocking
    let state_read = match app_state.try_read() {
        Some(state) => state,
        None => {
            tracing::warn!("Failed to acquire read lock for sending message");
            eprintln!("Failed to acquire read lock for sending message");
            return;
        }
    };
    
    if let Some(conversation_id) = &state_read.ai_chat.conversation_id {
        if let Some(token) = &state_read.auth_token {
            // Get current user info
            let author = state_read.current_user.as_ref()
                .map(|u| u.username.clone())
                .unwrap_or_else(|| "You".to_string());
            let author_id = state_read.current_user.as_ref()
                .map(|u| u.id)
                .unwrap_or(0);
            
            let message = shared::dto::messaging::Message::new(text.clone(), author.clone(), author_id);
            
            tracing::info!(
                conversation_id = %conversation_id,
                author = %author,
                author_id = author_id,
                message_length = text.len(),
                "Sending message to AI bot"
            );
            
            let mut braid_client = crate::services::braid_client::BraidClient::new(
                conversation_id.clone(),
                token.clone(),
            );
            
            let message_clone = message.clone();
            let conversation_id_clone = conversation_id.clone();
            let state_clone = app_state.clone();
            
            // Set typing indicator (use try_write to avoid blocking)
            {
                if let Some(mut state) = app_state.try_write() {
                    state.ai_chat.ai_typing = true;
                    state.ai_chat.message_input.clear();
                    tracing::debug!("Set AI typing indicator to true");
                }
            }
            
            drop(state_read); // Release read lock before spawning async task
            
            tokio::spawn(async move {
                tracing::debug!(
                    conversation_id = %conversation_id_clone,
                    "Calling braid_client.send_message()"
                );
                match braid_client.send_message(message_clone).await {
                    Ok(_) => {
                        tracing::info!(
                            conversation_id = %conversation_id_clone,
                            "Message sent successfully to AI bot - waiting for response via SSE"
                        );
                        // Message sent successfully
                        // The SSE subscription will update the UI with the new message
                        // and the AI response will come through the subscription
                    }
                    Err(e) => {
                        tracing::error!(
                            conversation_id = %conversation_id_clone,
                            error = %e,
                            "Failed to send message to AI bot"
                        );
                        eprintln!("Failed to send message to AI: {}", e);
                        // Use try_write to avoid blocking
                        if let Some(mut state) = state_clone.try_write() {
                            state.ai_chat.ai_typing = false;
                            state.pending_notifications.push((
                                "error".to_string(),
                                format!("Failed to send message: {}", e),
                            ));
                            tracing::debug!("Cleared AI typing indicator due to error");
                        }
                    }
                }
            });
        } else {
            tracing::warn!("Cannot send message - no auth token available");
        }
    } else {
        tracing::warn!("Cannot send message - no conversation ID available");
    }
}

