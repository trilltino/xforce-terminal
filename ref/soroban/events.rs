use serde::{Deserialize, Serialize};
use soroban_client::xdr::{ScVal, Limits, WriteXdr, ReadXdr};
use tracing::debug;

/// Pagination options for event queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pagination {
    /// Start from this ledger sequence (inclusive)
    From(u32),
    /// Start and end ledger sequences (start inclusive, end exclusive)
    FromTo(u32, u32),
    /// Continue from this cursor (from previous response)
    Cursor(String),
}

/// Event type filter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    /// Only contract events
    Contract,
    /// Only system events
    System,
    /// Only diagnostic events (protocol 23+: no longer returned)
    Diagnostic,
    /// All event types
    All,
}

impl EventType {
    pub fn as_str(&self) -> Option<&'static str> {
        match self {
            EventType::Contract => Some("contract"),
            EventType::System => Some("system"),
            EventType::Diagnostic => Some("diagnostic"),
            EventType::All => None,
        }
    }
}

/// Topic matcher for event filtering
#[derive(Debug, Clone)]
pub enum Topic {
    /// Match this specific ScVal
    Val(ScVal),
    /// Match any single topic
    Any,
    /// Match any topics including more topics (greedy, must be last)
    Greedy,
}

impl Topic {
    /// Convert to XDR base64 string for RPC
    pub fn to_xdr_string(&self) -> String {
        match self {
            Topic::Val(sc_val) => sc_val
                .to_xdr_base64(Limits::none())
                .unwrap_or_else(|_| "*".to_string()),
            Topic::Any => "*".to_string(),
            Topic::Greedy => "**".to_string(),
        }
    }
}

/// Event filter builder for querying contract events
#[derive(Debug, Clone)]
pub struct EventFilter {
    event_type: EventType,
    contract_ids: Vec<String>,
    topics: Vec<Vec<Topic>>,
}

impl EventFilter {
    /// Create a new event filter for the given event type
    pub fn new(event_type: EventType) -> Self {
        debug!("[EVENT_FILTER] Creating new filter for type: {:?}", event_type);
        Self {
            event_type,
            contract_ids: Vec::new(),
            topics: Vec::new(),
        }
    }

    /// Add a contract ID to filter (max 5 per request)
    pub fn contract(mut self, contract_id: impl Into<String>) -> Self {
        let id = contract_id.into();
        debug!("[EVENT_FILTER] Adding contract ID: {}", id);
        self.contract_ids.push(id);
        self
    }

    /// Add topic filters (max 5 per request)
    pub fn topic(mut self, filter: Vec<Topic>) -> Self {
        debug!("[EVENT_FILTER] Adding topic filter with {} topics", filter.len());
        self.topics.push(filter);
        self
    }

    /// Get event type as Option<String> for RPC
    pub fn event_type(&self) -> Option<String> {
        self.event_type.as_str().map(|s| s.to_string())
    }

    /// Get contract IDs list
    pub fn contracts(&self) -> Vec<String> {
        self.contract_ids.clone()
    }

    /// Get topics as Vec<Vec<String>> for RPC
    pub fn topics(&self) -> Vec<Vec<String>> {
        self.topics
            .iter()
            .map(|topic_vec| {
                topic_vec
                    .iter()
                    .map(|t| t.to_xdr_string())
                    .collect()
            })
            .collect()
    }

    /// Convert to JSON value for RPC request
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "type": self.event_type(),
            "contractIds": self.contracts(),
            "topics": self.topics(),
        })
    }
}

/// Response from get_events RPC call
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetEventsResponse {
    /// Events found for the filter
    pub events: Vec<EventResponse>,
    /// Cursor for next page (if more events available)
    pub cursor: Option<String>,
    /// Latest ledger sequence at time of request
    pub latest_ledger: u64,
    /// Oldest ledger sequence in RPC storage
    pub oldest_ledger: Option<u64>,
    /// Unix timestamp of latest ledger close
    pub latest_ledger_close_time: Option<String>,
    /// Unix timestamp of oldest ledger close
    pub oldest_ledger_close_time: Option<String>,
}

/// Individual event in response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventResponse {
    /// Event type
    #[serde(rename = "type")]
    pub event_type: String,
    /// Ledger sequence number
    pub ledger: u32,
    /// Unix timestamp of ledger close
    pub ledger_closed_at: String,
    /// Contract ID that emitted the event
    pub contract_id: String,
    /// Unique event ID (for pagination)
    pub id: String,
    /// Paging token for cursor-based pagination
    pub paging_token: String,
    /// Topic values (base64 encoded XDR)
    pub topic: Vec<String>,
    /// Event value (base64 encoded XDR)
    pub value: String,
    /// Whether event is in diagnostic events
    pub in_successful_contract_call: bool,
    /// Transaction hash that emitted this event
    pub transaction_hash: Option<String>,
}

impl GetEventsResponse {
    /// Check if there are more events to fetch
    pub fn has_more(&self) -> bool {
        self.cursor.is_some()
    }

    /// Get the cursor for the next page
    pub fn next_cursor(&self) -> Option<String> {
        self.cursor.clone()
    }

    /// Get total events in this response
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

impl EventResponse {
    /// Parse topic at index as ScVal
    pub fn parse_topic(&self, index: usize) -> Option<ScVal> {
        self.topic.get(index).and_then(|topic_xdr| {
            ScVal::from_xdr_base64(topic_xdr, Limits::none()).ok()
        })
    }

    /// Parse value as ScVal
    pub fn parse_value(&self) -> Option<ScVal> {
        ScVal::from_xdr_base64(&self.value, Limits::none()).ok()
    }

    /// Get all topics as ScVal
    pub fn parse_all_topics(&self) -> Vec<Option<ScVal>> {
        self.topic
            .iter()
            .map(|topic_xdr| ScVal::from_xdr_base64(topic_xdr, Limits::none()).ok())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_filter_builder() {
        let filter = EventFilter::new(EventType::Contract)
            .contract("CCFF5EA2CKR6VTHUTEKN7LNA26EPRSLZ6ZVBZFI2TRNTTD5C24BOKUIF")
            .topic(vec![Topic::Any, Topic::Any]);

        assert_eq!(filter.event_type(), Some("contract".to_string()));
        assert_eq!(filter.contracts().len(), 1);
        assert_eq!(filter.topics().len(), 1);
    }

    #[test]
    fn test_pagination() {
        let from = Pagination::From(1000);
        let from_to = Pagination::FromTo(1000, 2000);
        let cursor = Pagination::Cursor("cursor123".to_string());

        match from {
            Pagination::From(seq) => assert_eq!(seq, 1000),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_topic_xdr_string() {
        assert_eq!(Topic::Any.to_xdr_string(), "*");
        assert_eq!(Topic::Greedy.to_xdr_string(), "**");
    }
}
