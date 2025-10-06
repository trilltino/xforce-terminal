use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Contract operation to be queued
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractOperation {
    pub id: String,
    pub contract_id: String,
    pub function_name: String,
    pub source_account: String,
    pub signed_xdr: Option<String>,
    pub priority: OperationPriority,
    pub max_retries: u32,
    pub retry_count: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OperationPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl ContractOperation {
    pub fn new(
        contract_id: String,
        function_name: String,
        source_account: String,
        signed_xdr: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            contract_id,
            function_name,
            source_account,
            signed_xdr,
            priority: OperationPriority::Normal,
            max_retries: 3,
            retry_count: 0,
            created_at: chrono::Utc::now(),
        }
    }

    pub fn with_priority(mut self, priority: OperationPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
}

#[derive(Debug, Clone)]
pub enum QueueMessage {
    Submit(ContractOperation),
    Status(String),
    Shutdown,
}

#[derive(Debug, Clone)]
pub enum QueueResult {
    Success { operation_id: String, result: String },
    Retry { operation_id: String, attempt: u32 },
    Failed { operation_id: String, error: String },
}

/// Async queue for contract operations with retry logic
pub struct ContractQueue {
    tx: mpsc::UnboundedSender<QueueMessage>,
    result_rx: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<QueueResult>>>,
}

impl ContractQueue {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<QueueMessage>();
        let (result_tx, result_rx) = mpsc::unbounded_channel::<QueueResult>();

        // Clone tx for use in retry logic
        let tx_clone = tx.clone();

        // Spawn worker task
        tokio::spawn(async move {
            info!("🚀 Contract queue worker started");

            while let Some(message) = rx.recv().await {
                match message {
                    QueueMessage::Submit(mut operation) => {
                        info!("📥 Processing operation {} (priority: {:?})", operation.id, operation.priority);

                        // Simulate contract submission (replace with actual logic)
                        let result = Self::process_operation(&operation).await;

                        match result {
                            Ok(success_result) => {
                                info!("✅ Operation {} completed successfully", operation.id);
                                let _ = result_tx.send(QueueResult::Success {
                                    operation_id: operation.id.clone(),
                                    result: success_result,
                                });
                            }
                            Err(error) if operation.can_retry() => {
                                operation.increment_retry();
                                warn!("⚠️  Operation {} failed (attempt {}/{}): {}",
                                    operation.id, operation.retry_count, operation.max_retries, error);

                                // Exponential backoff
                                let delay = Duration::from_secs(2u64.pow(operation.retry_count));
                                sleep(delay).await;

                                // Retry
                                let _ = result_tx.send(QueueResult::Retry {
                                    operation_id: operation.id.clone(),
                                    attempt: operation.retry_count,
                                });

                                // Re-queue for retry
                                let _ = tx_clone.send(QueueMessage::Submit(operation));
                            }
                            Err(error) => {
                                error!("❌ Operation {} failed permanently: {}", operation.id, error);
                                let _ = result_tx.send(QueueResult::Failed {
                                    operation_id: operation.id.clone(),
                                    error,
                                });
                            }
                        }
                    }
                    QueueMessage::Status(operation_id) => {
                        info!("📊 Status check for operation: {}", operation_id);
                    }
                    QueueMessage::Shutdown => {
                        info!("🛑 Queue worker shutting down");
                        break;
                    }
                }
            }
        });

        Self {
            tx,
            result_rx: Arc::new(tokio::sync::Mutex::new(result_rx)),
        }
    }

    /// Submit operation to queue
    pub async fn submit(&self, operation: ContractOperation) -> Result<String, String> {
        let operation_id = operation.id.clone();

        self.tx
            .send(QueueMessage::Submit(operation))
            .map_err(|e| format!("Failed to queue operation: {}", e))?;

        info!("📤 Operation {} queued successfully", operation_id);
        Ok(operation_id)
    }

    /// Get next result from queue
    pub async fn next_result(&self) -> Option<QueueResult> {
        let mut rx = self.result_rx.lock().await;
        rx.recv().await
    }

    /// Shutdown the queue
    pub async fn shutdown(&self) -> Result<(), String> {
        self.tx
            .send(QueueMessage::Shutdown)
            .map_err(|e| format!("Failed to shutdown queue: {}", e))
    }

    /// Process a single operation (placeholder - implement actual contract logic)
    async fn process_operation(operation: &ContractOperation) -> Result<String, String> {
        // TODO: Replace with actual Stellar contract submission logic
        // For now, simulate processing time and success/failure

        sleep(Duration::from_millis(100)).await;

        // Simulate 90% success rate
        if rand::random::<f32>() < 0.9 {
            Ok(format!("Transaction submitted successfully for {}", operation.function_name))
        } else {
            Err(format!("Simulated error for {}", operation.function_name))
        }
    }
}

impl Default for ContractQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct QueueStats {
    pub pending_operations: usize,
    pub completed_operations: usize,
    pub failed_operations: usize,
    pub retry_operations: usize,
}
