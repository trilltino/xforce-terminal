// Transaction simulation types and functionality for Soroban contracts
//
// This module provides types and functionality for simulating transactions before submission,
// allowing you to:
// - Validate transaction logic without fees
// - Estimate accurate resource costs
// - Detect required restorations
// - Preview transaction results

use serde::{Deserialize, Serialize};
use soroban_client::xdr::{ScVal, Limits, ReadXdr, SorobanAuthorizationEntry};

/// Configuration for how resources will be calculated when simulating transactions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SimulationOptions {
    /// Allow this many extra instructions when budgeting resources
    #[serde(default)]
    pub cpu_instructions: u64,
    /// The auth mode to apply to the simulation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_mode: Option<AuthMode>,
}

/// Select the auth mode to apply to the simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMode {
    /// Always enforcement mode, even with an empty list of auths
    Enforce,
    /// Always recording mode, even with a non-empty list of auths
    Record,
}

impl From<AuthMode> for &str {
    fn from(mode: AuthMode) -> Self {
        match mode {
            AuthMode::Enforce => "enforce",
            AuthMode::Record => "record",
        }
    }
}

/// Response from simulateTransaction RPC call
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulateTransactionResponse {
    /// The sequence number of the latest ledger known to Stellar RPC at the time it handled the request
    pub latest_ledger: u32,

    /// (optional) Recommended minimum resource fee to add when submitting the transaction.
    /// This fee is to be added on top of the Stellar network fee.
    /// Not present in case of error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_resource_fee: Option<String>,

    /// (optional) Details about why the invoke host function call failed.
    /// Only present if the transaction simulation failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// (optional) Results from the host function invocation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<Vec<RawSimulateHostFunctionResult>>,

    /// (optional) Recommended Soroban Transaction Data (refundable fee and resource usage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_data: Option<String>,

    /// (optional) Indicates archived ledger entries that need restoration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restore_preamble: Option<RestorePreamble>,

    /// (optional) Array of serialized events emitted during simulation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<String>>,

    /// (optional) State changes that would occur if transaction is submitted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_changes: Option<Vec<RawStateChanges>>,
}

impl SimulateTransactionResponse {
    /// Extract the result value and authorization entries (if successful)
    pub fn to_result(&self) -> Option<(ScVal, Vec<SorobanAuthorizationEntry>)> {
        if let Some(results) = self.results.as_ref() {
            if results.is_empty() {
                return None;
            }

            let auth: Vec<SorobanAuthorizationEntry> = results[0]
                .auth
                .iter()
                .filter_map(|e| SorobanAuthorizationEntry::from_xdr_base64(e, Limits::none()).ok())
                .collect();

            let ret_val = ScVal::from_xdr_base64(&results[0].xdr, Limits::none()).ok()?;

            Some((ret_val, auth))
        } else {
            None
        }
    }

    /// Check if the simulation was successful
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }

    /// Check if restoration is required before submission
    pub fn needs_restoration(&self) -> bool {
        self.restore_preamble.is_some()
    }

    /// Get the estimated minimum resource fee
    pub fn get_min_resource_fee(&self) -> Option<u64> {
        self.min_resource_fee
            .as_ref()
            .and_then(|fee| fee.parse::<u64>().ok())
    }
}

/// Raw result from host function simulation
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawSimulateHostFunctionResult {
    /// Authorization entries (base64 XDR)
    pub auth: Vec<String>,
    /// Return value (base64 XDR)
    pub xdr: String,
}

/// Information about required restoration
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorePreamble {
    /// Minimum resource fee for restoration transaction
    pub min_resource_fee: String,
    /// Transaction data for restoration operation
    pub transaction_data: String,
}

/// State change detected during simulation
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawStateChanges {
    /// Type of state change
    #[serde(rename = "type")]
    pub kind: StateChangeKind,
    /// Ledger key (base64 XDR)
    pub key: String,
    /// State before (base64 XDR, if exists)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// State after (base64 XDR)
    pub after: String,
}

/// Type of state change
#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StateChangeKind {
    Created,
    Updated,
    Deleted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_options_default() {
        let options = SimulationOptions::default();
        assert_eq!(options.cpu_instructions, 0);
        assert!(options.auth_mode.is_none());
    }

    #[test]
    fn test_auth_mode_conversion() {
        let enforce: &str = AuthMode::Enforce.into();
        assert_eq!(enforce, "enforce");

        let record: &str = AuthMode::Record.into();
        assert_eq!(record, "record");
    }

    #[test]
    fn test_simulation_response_success() {
        let response = SimulateTransactionResponse {
            latest_ledger: 12345,
            min_resource_fee: Some("100".to_string()),
            error: None,
            results: None,
            transaction_data: None,
            restore_preamble: None,
            events: None,
            state_changes: None,
        };

        assert!(response.is_success());
        assert!(!response.needs_restoration());
        assert_eq!(response.get_min_resource_fee(), Some(100));
    }

    #[test]
    fn test_simulation_response_error() {
        let response = SimulateTransactionResponse {
            latest_ledger: 12345,
            min_resource_fee: None,
            error: Some("Transaction failed".to_string()),
            results: None,
            transaction_data: None,
            restore_preamble: None,
            events: None,
            state_changes: None,
        };

        assert!(!response.is_success());
        assert!(!response.needs_restoration());
    }

    #[test]
    fn test_simulation_response_needs_restoration() {
        let response = SimulateTransactionResponse {
            latest_ledger: 12345,
            min_resource_fee: Some("100".to_string()),
            error: None,
            results: None,
            transaction_data: None,
            restore_preamble: Some(RestorePreamble {
                min_resource_fee: "200".to_string(),
                transaction_data: "base64data".to_string(),
            }),
            events: None,
            state_changes: None,
        };

        assert!(response.is_success());
        assert!(response.needs_restoration());
    }
}
