//! # Messaging Screen
//!
//! Messaging interface with friends list, friend requests, and direct messaging.
//! Implements a Bloomberg Terminal-style messaging interface.

use egui;
use crate::app::{AppState, AppLike};
use crate::ui::theme::Theme;
use chrono::DateTime;
use std::sync::Arc;
use parking_lot::RwLock;

/// Render messaging screen
pub fn render(ui: &mut egui::Ui, state: &AppState, app: &mut impl AppLike) {
    let theme = Theme::default();
    
    // Load friends list if not loaded yet
    if state.messaging.friends.is_empty() && state.messaging.incoming_requests.is_empty() && state.messaging.outgoing_requests.is_empty() {
        if let Some(api_client) = &state.api_client {
            if let Some(token) = &state.auth_token {
                let client = api_client.clone();
                let token = token.clone();
                let state_clone = app.state().clone();
                tokio::spawn(async move {
                    match client.get_friends(&token).await {
                        Ok(friends_list) => {
                            let mut state = state_clone.write();
                            state.messaging.friends = friends_list.friends;
                            state.messaging.incoming_requests = friends_list.incoming_requests;
                            state.messaging.outgoing_requests = friends_list.outgoing_requests;
                        }
                        Err(e) => {
                            eprintln!("Failed to load friends: {}", e);
                        }
                    }
                });
            }
        }
    }
    
    // Main layout: Friends list (30%) | Chat panel (70%)
    ui.columns(2, |columns| {
        // Left panel: Friends list and requests
        columns[0].vertical(|ui| {
            ui.set_width(300.0);
            render_friends_panel(ui, state, app, &theme);
        });
        
        // Right panel: Active conversation
        columns[1].vertical(|ui| {
            render_chat_panel(ui, state, app, &theme);
        });
    });
}

/// Render friends panel (left side)
fn render_friends_panel(ui: &mut egui::Ui, state: &AppState, app: &mut impl AppLike, theme: &Theme) {
    use crate::ui::widgets::layouts;
    
    // Clone app.state at the beginning to avoid borrow conflicts in closures
    let app_state = app.state().clone();
    
    layouts::render_panel(ui, None, |ui| {
        ui.heading("Friends");
        
        ui.separator();
        
        // Search bar
        ui.horizontal(|ui| {
            let response = ui.text_edit_singleline(&mut app_state.write().messaging.search_query);
            if response.lost_focus() && response.ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                // Trigger search
                let query = state.messaging.search_query.clone();
                if !query.is_empty() {
                    if let Some(api_client) = &state.api_client {
                        if let Some(token) = &state.auth_token {
                            let client = api_client.clone();
                            let token = token.clone();
                            let state_clone = app_state.clone();
                            tokio::spawn(async move {
                                match client.search_users(&token, &query).await {
                                    Ok(results) => {
                                        let mut state = state_clone.write();
                                        state.messaging.search_results = results.users;
                                    }
                                    Err(e) => {
                                        eprintln!("Search failed: {}", e);
                                    }
                                }
                            });
                        }
                    }
                }
            }
            if ui.button("Search").clicked() {
                let query = state.messaging.search_query.clone();
                if !query.is_empty() {
                    if let Some(api_client) = &state.api_client {
                        if let Some(token) = &state.auth_token {
                            let client = api_client.clone();
                            let token = token.clone();
                            let state_clone = app_state.clone();
                            tokio::spawn(async move {
                                match client.search_users(&token, &query).await {
                                    Ok(results) => {
                                        let mut state = state_clone.write();
                                        state.messaging.search_results = results.users;
                                    }
                                    Err(e) => {
                                        eprintln!("Search failed: {}", e);
                                    }
                                }
                            });
                        }
                    }
                }
            }
        });
        
        // Show search results
        if !state.messaging.search_results.is_empty() {
            ui.separator();
            ui.label("Search Results:");
            for user in &state.messaging.search_results {
                ui.horizontal(|ui| {
                    ui.label(&user.username);
                    if ui.button("Add Friend").clicked() {
                        if let Some(api_client) = &state.api_client {
                            if let Some(token) = &state.auth_token {
                                let client = api_client.clone();
                                let token = token.clone();
                                let user_id = user.id;
                                let state_clone = app_state.clone();
                                tokio::spawn(async move {
                                    match client.send_friend_request(&token, user_id).await {
                                        Ok(_) => {
                                            // Refresh friends list
                                            if let Ok(friends_list) = client.get_friends(&token).await {
                                                let mut state = state_clone.write();
                                                state.messaging.friends = friends_list.friends;
                                                state.messaging.incoming_requests = friends_list.incoming_requests;
                                                state.messaging.outgoing_requests = friends_list.outgoing_requests;
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to send friend request: {}", e);
                                        }
                                    }
                                });
                            }
                        }
                    }
                });
            }
        }
        
        ui.separator();
        
        // Pending requests section
        if !state.messaging.incoming_requests.is_empty() {
            ui.label("Incoming Requests:");
            for request in &state.messaging.incoming_requests {
                ui.horizontal(|ui| {
                    ui.label(&request.sender_username);
                    if ui.button("Accept").clicked() {
                        if let Some(api_client) = &state.api_client {
                            if let Some(token) = &state.auth_token {
                                let client = api_client.clone();
                                let token = token.clone();
                                let request_id = request.id;
                                let state_clone = app_state.clone();
                                tokio::spawn(async move {
                                    if client.accept_friend_request(&token, request_id).await.is_ok() {
                                        // Refresh friends list
                                        if let Ok(friends_list) = client.get_friends(&token).await {
                                            let mut state = state_clone.write();
                                            state.messaging.friends = friends_list.friends;
                                            state.messaging.incoming_requests = friends_list.incoming_requests;
                                            state.messaging.outgoing_requests = friends_list.outgoing_requests;
                                        }
                                    }
                                });
                            }
                        }
                    }
                    if ui.button("Reject").clicked() {
                        if let Some(api_client) = &state.api_client {
                            if let Some(token) = &state.auth_token {
                                let client = api_client.clone();
                                let token = token.clone();
                                let request_id = request.id;
                                let state_clone = app_state.clone();
                                tokio::spawn(async move {
                                    if client.reject_friend_request(&token, request_id).await.is_ok() {
                                        // Refresh friends list
                                        if let Ok(friends_list) = client.get_friends(&token).await {
                                            let mut state = state_clone.write();
                                            state.messaging.friends = friends_list.friends;
                                            state.messaging.incoming_requests = friends_list.incoming_requests;
                                            state.messaging.outgoing_requests = friends_list.outgoing_requests;
                                        }
                                    }
                                });
                            }
                        }
                    }
                });
            }
            ui.separator();
        }
        
        // Friends list
        ui.label(format!("Friends ({})", state.messaging.friends.len()));
        
        if state.messaging.friends.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No friends yet. Search for users to add as friends.");
            });
        } else {
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    for friend in &state.messaging.friends {
                        let is_selected = state.messaging.selected_user_id == Some(friend.user_id);
                        
                        let button = if is_selected {
                            egui::Button::new(format!("{} {}", 
                                if friend.unread_count > 0 {
                                    format!("[{}] ", friend.unread_count)
                                } else {
                                    String::new()
                                },
                                friend.username
                            )).fill(theme.selected)
                        } else {
                            egui::Button::new(format!("{} {}", 
                                if friend.unread_count > 0 {
                                    format!("[{}] ", friend.unread_count)
                                } else {
                                    String::new()
                                },
                                friend.username
                            ))
                        };
                        
                        if ui.add(button).clicked() {
                            let mut state_write = app_state.write();
                            state_write.messaging.selected_user_id = Some(friend.user_id);
                            
                            // Compute conversation ID from user IDs
                            if let Some(current_user) = &state_write.current_user {
                                let conversation_id = format!("{}:{}",
                                    std::cmp::min(current_user.id, friend.user_id),
                                    std::cmp::max(current_user.id, friend.user_id)
                                );
                                // Clone conversation_id before moving it
                                let conversation_id_clone = conversation_id.clone();
                                state_write.messaging.active_conversation_id = Some(conversation_id);
                                
                                // Start SSE subscription for this conversation
                                let token = state_write.auth_token.clone();
                                let state_clone = app_state.clone();
                                
                                if let Some(token) = token {
                                    drop(state_write);
                                    tokio::spawn(async move {
                                        // Subscribe to conversation updates
                                        let mut braid_client = crate::services::braid_client::BraidClient::new(
                                            conversation_id_clone.clone(),
                                            token,
                                        );
                                        
                                        match braid_client.subscribe().await {
                                            Ok(mut rx) => {
                                                while let Some((messages, _version)) = rx.recv().await {
                                                    let mut state = state_clone.write();
                                                    state.messaging.messages.insert(conversation_id_clone.clone(), messages);
                                                    drop(state);
                                                    // UI will update on next frame
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to subscribe to conversation: {}", e);
                                            }
                                        }
                                    });
                                }
                            }
                        }
                    }
                });
        }
    });
}

/// Render chat panel (right side)
fn render_chat_panel(ui: &mut egui::Ui, state: &AppState, app: &mut impl AppLike, _theme: &Theme) {
    use crate::ui::widgets::layouts;
    
    // Clone app.state at the beginning to avoid borrow conflicts in closures
    let app_state = app.state().clone();
    
    layouts::render_panel(ui, None, |ui| {
        if let Some(conversation_id) = &state.messaging.active_conversation_id {
            // Show conversation header with friend's name
            if let Some(user_id) = state.messaging.selected_user_id {
                if let Some(friend) = state.messaging.friends.iter().find(|f| f.user_id == user_id) {
                    ui.heading(format!("Conversation with {}", friend.username));
                } else {
                    ui.heading("Conversation");
                }
            } else {
                ui.heading("Conversation");
            }
            
            ui.separator();
            
            // Message history area
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(400.0)
                .show(ui, |ui| {
                    if let Some(messages) = state.messaging.messages.get(conversation_id) {
                        for message in messages {
                            ui.horizontal(|ui| {
                                // Message bubble
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("{}:", message.author));
                                        ui.label(&message.text);
                                    });
                                    // Timestamp
                                    if let Ok(timestamp) = DateTime::parse_from_rfc3339(&message.timestamp) {
                                        ui.label(format!("{}", timestamp.format("%H:%M")));
                                    }
                                });
                            });
                            ui.add_space(5.0);
                        }
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label("No messages yet. Start the conversation!");
                        });
                    }
                    
                    // Typing indicator
                    if let Some((_user_id, username)) = state.messaging.typing_indicators.get(conversation_id) {
                        ui.label(format!("{} is typing...", username));
                    }
                });
            
            ui.separator();
            
            // Message input area
            ui.horizontal(|ui| {
                let mut state_write = app_state.write();
                let text_edit = egui::TextEdit::singleline(&mut state_write.messaging.message_input)
                    .desired_width(f32::INFINITY)
                    .hint_text("Type a message...");
                
                let response = ui.add(text_edit);
                let message_text = state_write.messaging.message_input.clone();
                drop(state_write);
                
                if response.lost_focus() && response.ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if !message_text.trim().is_empty() {
                        send_message(app_state.clone(), conversation_id.clone(), message_text.clone());
                    }
                }
                
                if ui.button("Send").clicked() {
                    if !message_text.trim().is_empty() {
                        send_message(app_state.clone(), conversation_id.clone(), message_text.clone());
                    }
                }
            });
        } else {
            // Empty state
            ui.centered_and_justified(|ui| {
                ui.heading("Select a friend to start messaging");
                ui.label("Choose a friend from the list on the left to start a conversation.");
            });
        }
    });
}

/// Send a message
fn send_message(app_state: Arc<RwLock<AppState>>, conversation_id: String, text: String) {
    let state_read = app_state.read();
    
    if let Some(token) = &state_read.auth_token {
        // Get current user info
        let author = state_read.current_user.as_ref()
            .map(|u| u.username.clone())
            .unwrap_or_else(|| "You".to_string());
        let author_id = state_read.current_user.as_ref()
            .map(|u| u.id)
            .unwrap_or(0);
        
        let message = shared::dto::messaging::Message::new(text, author, author_id);
        
        let mut braid_client = crate::services::braid_client::BraidClient::new(
            conversation_id.clone(),
            token.clone(),
        );
        
        let message_clone = message.clone();
        let state_clone = app_state.clone();
        
        tokio::spawn(async move {
            match braid_client.send_message(message_clone).await {
                Ok(_) => {
                    // Message sent successfully - clear input
                    let mut state = state_clone.write();
                    state.messaging.message_input.clear();
                    // The SSE subscription will update the UI with the new message
                }
                Err(e) => {
                    eprintln!("Failed to send message: {}", e);
                    let mut state = state_clone.write();
                    state.pending_notifications.push((
                        "error".to_string(),
                        format!("Failed to send message: {}", e),
                    ));
                }
            }
        });
    }
}

