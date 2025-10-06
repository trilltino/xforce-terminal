use soroban_client::xdr::ScVal;
use soroban_client::address::AddressTrait;
use crate::error::{AppError, Result};

/// Represents a Soroban contract function that can be called
#[derive(Debug, Clone)]
pub enum ContractFunction {
    // Add specific contract functions as needed
    GetBalance { address: String },
    GetReserves,
    Swap { amount_in: i128, min_amount_out: i128 },
    Custom { name: String, params: Vec<ScVal> },
}

impl ContractFunction {
    pub fn name(&self) -> &str {
        match self {
            ContractFunction::GetBalance { .. } => "balance",
            ContractFunction::GetReserves => "get_rsrvs",
            ContractFunction::Swap { .. } => "swap",
            ContractFunction::Custom { name, .. } => name,
        }
    }

    pub fn signature(&self) -> String {
        match self {
            ContractFunction::GetBalance { .. } => "balance(address: Address) -> i128".to_string(),
            ContractFunction::GetReserves => "get_rsrvs() -> (i128, i128)".to_string(),
            ContractFunction::Swap { .. } => "swap(to: Address, buy_a: bool, out: i128, in_max: i128)".to_string(),
            ContractFunction::Custom { name, .. } => format!("{}(...)", name),
        }
    }

    pub fn description(&self) -> &str {
        match self {
            ContractFunction::GetBalance { .. } => "Get token balance for an address",
            ContractFunction::GetReserves => "Get liquidity pool reserves",
            ContractFunction::Swap { .. } => "Execute token swap",
            ContractFunction::Custom { .. } => "Custom contract function",
        }
    }

    pub fn to_scval_params(&self) -> Result<Vec<ScVal>> {
        match self {
            ContractFunction::GetBalance { address } => {
                let addr = soroban_client::address::Address::new(address)
                    .map_err(|e| AppError::InvalidInput(format!("Invalid address: {:?}", e)))?;
                Ok(vec![addr.to_sc_val()
                    .map_err(|e| AppError::XdrEncoding(format!("Failed to convert address: {:?}", e)))?])
            }
            ContractFunction::GetReserves => Ok(vec![]),
            ContractFunction::Swap { amount_in, min_amount_out } => {
                Ok(vec![
                    i128_to_scval(*amount_in),
                    i128_to_scval(*min_amount_out),
                ])
            }
            ContractFunction::Custom { params, .. } => Ok(params.clone()),
        }
    }
}

// Helper to convert i128 to ScVal
fn i128_to_scval(value: i128) -> ScVal {
    use soroban_client::xdr::Int128Parts;
    let hi = (value >> 64) as i64;
    let lo = value as u64;
    ScVal::I128(Int128Parts { hi, lo })
}
