// Contract state querying functionality for Soroban contracts
//
// This module provides types and functionality for reading contract storage directly,
// allowing you to:
// - Read persistent storage (long-lived data)
// - Read temporary storage (short-lived data)
// - Verify contract state before transactions
// - Build dashboards from contract data
// - Access data without gas fees

use serde::{Deserialize, Serialize};
use soroban_client::xdr::{LedgerKey, LedgerEntryData, Limits, ReadXdr};

/// Storage durability type for contract data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Durability {
    /// Temporary storage - short-lived data (ledger-based TTL)
    /// Cannot be restored once expired
    /// Use for: session data, cache, ephemeral state
    Temporary,

    /// Persistent storage - long-lived data (instance-based TTL)
    /// Archived when TTL expires, can be restored
    /// Use for: user data, game state, permanent records
    Persistent,
}

impl Durability {
    /// Convert to XDR ContractDataDurability
    pub fn to_xdr(&self) -> soroban_client::xdr::ContractDataDurability {
        match self {
            Durability::Temporary => soroban_client::xdr::ContractDataDurability::Temporary,
            Durability::Persistent => soroban_client::xdr::ContractDataDurability::Persistent,
        }
    }

    /// Create from XDR ContractDataDurability
    pub fn from_xdr(xdr: soroban_client::xdr::ContractDataDurability) -> Self {
        match xdr {
            soroban_client::xdr::ContractDataDurability::Temporary => Durability::Temporary,
            soroban_client::xdr::ContractDataDurability::Persistent => Durability::Persistent,
        }
    }
}

/// Result from ledger entry query
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerEntryResult {
    /// Ledger sequence when this entry was last modified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_ledger_seq: Option<u32>,

    /// Ledger sequence when this entry will expire (TTL)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live_until_ledger_seq: Option<u32>,

    /// Ledger key (base64 XDR)
    pub key: String,

    /// Ledger entry data (base64 XDR)
    pub xdr: String,

    /// Extension data (base64 XDR)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ext_xdr: Option<String>,
}

impl LedgerEntryResult {
    /// Parse the ledger key from base64 XDR
    pub fn to_key(&self) -> Result<LedgerKey, String> {
        LedgerKey::from_xdr_base64(&self.key, Limits::none())
            .map_err(|e| format!("Failed to parse ledger key: {}", e))
    }

    /// Parse the ledger entry data from base64 XDR
    pub fn to_data(&self) -> Result<LedgerEntryData, String> {
        LedgerEntryData::from_xdr_base64(&self.xdr, Limits::none())
            .map_err(|e| format!("Failed to parse ledger entry data: {}", e))
    }

    /// Check if the entry has expired based on current ledger
    pub fn is_expired(&self, current_ledger: u32) -> bool {
        if let Some(live_until) = self.live_until_ledger_seq {
            current_ledger > live_until
        } else {
            false
        }
    }

    /// Get remaining TTL in ledgers
    pub fn remaining_ttl(&self, current_ledger: u32) -> Option<u32> {
        self.live_until_ledger_seq.map(|live_until| {
            if live_until > current_ledger {
                live_until - current_ledger
            } else {
                0
            }
        })
    }
}

/// Response from getLedgerEntries RPC call
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetLedgerEntriesResponse {
    /// Array of found ledger entries
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<Vec<LedgerEntryResult>>,

    /// Latest ledger sequence at query time
    pub latest_ledger: u32,
}

impl GetLedgerEntriesResponse {
    /// Get the first entry (convenience for single queries)
    pub fn first_entry(&self) -> Option<&LedgerEntryResult> {
        self.entries.as_ref().and_then(|e| e.first())
    }

    /// Get number of entries found
    pub fn entry_count(&self) -> usize {
        self.entries.as_ref().map(|e| e.len()).unwrap_or(0)
    }

    /// Check if any entries were found
    pub fn has_entries(&self) -> bool {
        self.entry_count() > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_durability_conversion() {
        let temp = Durability::Temporary;
        let temp_xdr = temp.to_xdr();
        let temp_back = Durability::from_xdr(temp_xdr);
        assert_eq!(temp, temp_back);

        let persist = Durability::Persistent;
        let persist_xdr = persist.to_xdr();
        let persist_back = Durability::from_xdr(persist_xdr);
        assert_eq!(persist, persist_back);
    }

    #[test]
    fn test_ledger_entry_expiry() {
        let entry = LedgerEntryResult {
            last_modified_ledger_seq: Some(100),
            live_until_ledger_seq: Some(200),
            key: "test".to_string(),
            xdr: "test".to_string(),
            ext_xdr: None,
        };

        assert!(!entry.is_expired(150));
        assert!(!entry.is_expired(200));
        assert!(entry.is_expired(201));

        assert_eq!(entry.remaining_ttl(150), Some(50));
        assert_eq!(entry.remaining_ttl(200), Some(0));
        assert_eq!(entry.remaining_ttl(201), Some(0));
    }

    #[test]
    fn test_ledger_entry_no_ttl() {
        let entry = LedgerEntryResult {
            last_modified_ledger_seq: Some(100),
            live_until_ledger_seq: None,
            key: "test".to_string(),
            xdr: "test".to_string(),
            ext_xdr: None,
        };

        assert!(!entry.is_expired(999999));
        assert_eq!(entry.remaining_ttl(100), None);
    }

    #[test]
    fn test_get_ledger_entries_response() {
        let response = GetLedgerEntriesResponse {
            entries: Some(vec![
                LedgerEntryResult {
                    last_modified_ledger_seq: Some(100),
                    live_until_ledger_seq: Some(200),
                    key: "key1".to_string(),
                    xdr: "data1".to_string(),
                    ext_xdr: None,
                },
                LedgerEntryResult {
                    last_modified_ledger_seq: Some(101),
                    live_until_ledger_seq: Some(201),
                    key: "key2".to_string(),
                    xdr: "data2".to_string(),
                    ext_xdr: None,
                },
            ]),
            latest_ledger: 150,
        };

        assert_eq!(response.entry_count(), 2);
        assert!(response.has_entries());
        assert_eq!(response.first_entry().unwrap().key, "key1");
    }

    #[test]
    fn test_empty_response() {
        let response = GetLedgerEntriesResponse {
            entries: None,
            latest_ledger: 150,
        };

        assert_eq!(response.entry_count(), 0);
        assert!(!response.has_entries());
        assert!(response.first_entry().is_none());
    }
}
