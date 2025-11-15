//! # Clear Users Utility
//!
//! This binary deletes all users from the database.
//!
//! **WARNING**: This is a destructive operation that cannot be undone.
//!
//! ## Usage
//!
//! ```bash
//! cargo run --package clear-users --bin clear_users
//! ```
//!
//! The program will:
//! 1. Connect to the database
//! 2. Count existing users
//! 3. Ask for confirmation
//! 4. Delete all users if confirmed
//! 5. Report the number of users deleted

use lib_core::{create_pool};
use lib_core::model::store::UserRepository;
use sqlx::query_as;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    
    println!("============================================");
    println!("  Clear Users Utility");
    println!("============================================");
    println!();
    println!("WARNING: This will delete ALL users from the database!");
    println!("This operation cannot be undone.");
    println!();

    // Connect to database
    println!("Connecting to database...");
    let pool = create_pool().await?;
    println!("Connected successfully.");
    println!();

    // Count existing users
    let user_count: (i64,) = query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await?;

    if user_count.0 == 0 {
        println!("No users found in the database.");
        println!("Nothing to delete.");
        return Ok(());
    }

    println!("Found {} user(s) in the database.", user_count.0);
    println!();

    // Ask for confirmation
    print!("Are you sure you want to delete all users? (yes/no): ");
    io::stdout().flush()?;

    let mut confirmation = String::new();
    io::stdin().read_line(&mut confirmation)?;
    let confirmation = confirmation.trim().to_lowercase();

    if confirmation != "yes" && confirmation != "y" {
        println!("Operation cancelled.");
        return Ok(());
    }

    println!();
    println!("Deleting all users...");

    // Delete all users
    let deleted_count = UserRepository::delete_all(&pool).await?;

    println!("Successfully deleted {} user(s).", deleted_count);
    println!();
    println!("Database cleared.");

    Ok(())
}

