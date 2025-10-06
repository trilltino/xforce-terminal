/// Global Tokio runtime for async HTTP operations
///
/// GTK uses glib's MainContext for async operations, but reqwest requires
/// a tokio runtime. This static runtime bridges the two by:
/// 1. Providing a tokio context for reqwest to execute in
/// 2. Using glib::idle_add_once to send results back to the main thread
///
/// Usage:
/// ```rust
/// use crate::utils::runtime::TOKIO_RT;
///
/// TOKIO_RT.spawn(async move {
///     let result = some_async_operation().await;
///     glib::idle_add_once(move || {
///         // Update UI on main thread
///     });
/// });
/// ```

use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

pub static TOKIO_RT: Lazy<Runtime> = Lazy::new(|| {
    Runtime::new().expect("Failed to create Tokio runtime for async HTTP operations")
});
