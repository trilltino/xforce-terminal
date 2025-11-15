//! # Friend Management API Client
//!
//! HTTP client methods for friend requests and friend management.

use super::client::ApiClient;
use shared::dto::messaging::*;

impl ApiClient {
    
    /// Send a friend request to another user
    pub async fn send_friend_request(&self, token: &str, receiver_id: i64) -> Result<FriendRequestResponse, String> {
        let url = format!("{}/api/friends/request", ApiClient::base_url());
        
        let request = FriendRequestRequest { receiver_id };
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
        
        if response.status().is_success() {
            response.json::<FriendRequestResponse>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("API error: {}", error_text))
        }
    }
    
    /// Accept a friend request
    pub async fn accept_friend_request(&self, token: &str, request_id: i64) -> Result<(), String> {
        let url = format!("{}/api/friends/accept/{}", ApiClient::base_url(), request_id);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
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
    
    /// Reject a friend request
    pub async fn reject_friend_request(&self, token: &str, request_id: i64) -> Result<(), String> {
        let url = format!("{}/api/friends/reject/{}", ApiClient::base_url(), request_id);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
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
    
    /// Block a user
    pub async fn block_user(&self, token: &str, user_id: i64) -> Result<(), String> {
        let url = format!("{}/api/friends/block/{}", ApiClient::base_url(), user_id);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
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
    
    /// Get friends list and pending requests
    pub async fn get_friends(&self, token: &str) -> Result<FriendsListResponse, String> {
        let url = format!("{}/api/friends", ApiClient::base_url());
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
        
        if response.status().is_success() {
            response.json::<FriendsListResponse>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("API error: {}", error_text))
        }
    }
    
    /// Search for users by username
    pub async fn search_users(&self, token: &str, query: &str) -> Result<UserSearchResponse, String> {
        let url = format!("{}/api/friends/search", ApiClient::base_url());
        
        let response = self.client
            .get(&url)
            .query(&[("query", query)])
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
        
        if response.status().is_success() {
            response.json::<UserSearchResponse>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("API error: {}", error_text))
        }
    }
}

