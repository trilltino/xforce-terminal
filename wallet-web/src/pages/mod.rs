//! Page modules - minimal wallet helper pages only

pub mod connect;
pub mod status;
pub mod wallet_setup;
pub mod transaction_sign;
pub mod about;

pub use connect::ConnectPage;
pub use status::StatusPage;
pub use wallet_setup::WalletSetupPage;
pub use transaction_sign::TransactionSignPage;
pub use about::AboutPage;
