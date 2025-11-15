//! File-based logging initialization

use super::config::DebugConfig;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use std::fs;

/// Initialize the logging system
///
/// Sets up file-based logging with:
/// - Daily log rotation for main debug log
/// - Optional realtime debug log (truncated on startup, for live monitoring)
/// - Structured output with trace IDs
/// - Non-blocking writes to prevent UI lag
/// - Panic hook integration for crash logging
///
/// Logs are written to `logs/terminal-debug.log` by default.
/// If realtime logging is enabled, also writes to `logs/debug-realtime.log`.
pub fn init() {
    let config = DebugConfig::from_env();

    // Create logs directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&config.log_dir) {
        eprintln!("Warning: Failed to create log directory: {}", e);
        return;
    }

    // Create file appender with daily rotation for main log
    let file_appender = tracing_appender::rolling::daily(&config.log_dir, "terminal-debug.log");
    let (non_blocking_main, _guard_main) = tracing_appender::non_blocking(file_appender);

    // Configure log filter from environment
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.log_level))
        .unwrap_or_else(|_| EnvFilter::new("terminal=info,warn"));

    // Build subscriber with file output for main log
    let file_layer = fmt::layer()
        .with_writer(non_blocking_main)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false); // No ANSI codes in log files

    let subscriber = tracing_subscriber::registry()
        .with(env_filter.clone())
        .with(file_layer);

    // Add realtime log layer if enabled
    if config.enable_realtime_log {
        // Truncate realtime log on startup for fresh session
        let realtime_path = config.log_dir.join("debug-realtime.log");
        if let Err(e) = fs::File::create(&realtime_path) {
            eprintln!("Warning: Failed to create realtime log file: {}", e);
        }

        // Create non-rotating appender for realtime log
        let realtime_appender = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&realtime_path)
            .expect("Failed to open realtime log file");

        let (non_blocking_realtime, _guard_realtime) = tracing_appender::non_blocking(realtime_appender);

        // More verbose format for realtime debugging
        let realtime_layer = fmt::layer()
            .with_writer(non_blocking_realtime)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_ansi(false)
            .pretty(); // Human-readable format

        let subscriber = subscriber.with(realtime_layer);

        // Initialize with both layers
        subscriber.init();

        // Keep guards alive
        std::mem::forget(_guard_realtime);
    } else {
        // Initialize with just main layer
        subscriber.init();
    }

    // Log initialization - this ensures the log file is written to immediately
    tracing::info!(
        log_file = %config.log_file.display(),
        log_level = %config.log_level,
        debug_ui = config.show_debug_ui,
        realtime_log = config.enable_realtime_log,
        trace_ids = config.enable_trace_ids,
        freeze_threshold_ms = config.freeze_threshold_ms,
        "Debug logging initialized"
    );
    
    // Force a flush to ensure the log file is written immediately
    // This helps the debug viewer detect the log file exists
    if config.enable_realtime_log {
        tracing::info!(
            realtime_log_path = %config.log_dir.join("debug-realtime.log").display(),
            "Realtime debug log file created and ready for monitoring"
        );
    }

    // Set up enhanced panic hook
    setup_panic_hook();

    // Keep the main guard alive for the lifetime of the program
    std::mem::forget(_guard_main);
}

/// Set up panic hook to log panics with full context
fn setup_panic_hook() {
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Print to stderr first (visible in console)
        eprintln!("\n!!!!! PANIC DETECTED !!!!!");
        eprintln!("Panic: {:?}", panic_info);
        
        // Get location
        let location = panic_info.location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        // Enhanced logging for TextStyle resolution panics
        if let Some(loc) = panic_info.location() {
            eprintln!("Location: {}:{}:{}", loc.file(), loc.line(), loc.column());
            
            if loc.file().contains("style.rs") && loc.line() == 112 {
                eprintln!("\n!!! TEXTSTYLE RESOLUTION PANIC !!!");
                eprintln!("This panic occurs when a TextStyle is not found in the text_styles map.");
                eprintln!("Common causes:");
                eprintln!("  1. Using TextStyle::Name(\"...\") without registering it");
                eprintln!("  2. Font initialization not called before theme application");
                eprintln!("  3. TextStyle used before setup_terminal_fonts() is called");
                eprintln!("\nCheck that:");
                eprintln!("  - FontConfig::setup_terminal_fonts() is called before theme application");
                eprintln!("  - Only built-in TextStyles (Heading, Body, Button, etc.) are used");
                eprintln!("  - Custom TextStyle names are registered before use");
            }
        }

        // Get message
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            eprintln!("Message: {}", s);
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            eprintln!("Message: {}", s);
            s.clone()
        } else {
            "unknown panic message".to_string()
        };

        // Print backtrace to stderr
        eprintln!("Backtrace:");
        let backtrace = std::backtrace::Backtrace::force_capture();
        eprintln!("{:?}", backtrace);
        eprintln!("!!!!! END PANIC !!!!!\n");

        // Also log to file with trace ID if available
        if let Some(trace_id) = super::trace_context::get_trace_id() {
            tracing::error!(
                trace_id = %trace_id,
                location = %location,
                message = %message,
                "!!!!! APPLICATION PANIC !!!!!"
            );
        } else {
            tracing::error!(
                location = %location,
                message = %message,
                "!!!!! APPLICATION PANIC !!!!!"
            );
        }

        // Log backtrace to file
        tracing::error!(
            backtrace = %backtrace,
            "Panic backtrace"
        );

        // Call default panic handler (shows Windows error dialog, etc.)
        default_panic(panic_info);
    }));
}
