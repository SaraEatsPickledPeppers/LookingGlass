//! Structured logging for Looking Glass.
//!
//! Provides JSON-formatted logs to ~/.local/share/LookingGlass/logs/
//! with timestamps, level, module, and message fields.

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt::Layer,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// Guard to keep the logging worker alive for the application lifetime.
pub struct LoggingGuard {
    _guard: WorkerGuard,
}

impl LoggingGuard {
    /// Initialize structured logging with file output.
    ///
    /// - `verbose`: enables INFO level logging
    /// - `debug`: enables DEBUG level logging (implies verbose)
    /// - Returns a guard that must be kept alive for the duration of logging
    pub fn init(verbose: bool, debug: bool) -> Result<Self> {
        // Determine log directory: ${XDG_DATA_HOME:-$HOME/.local/share}/LookingGlass/logs/
        let log_dir = get_log_directory()?;

        // Create log directory if it doesn't exist
        fs::create_dir_all(&log_dir).with_context(|| format!("Creating log directory: {}", log_dir.display()))?;

        // Generate log file name with timestamp: looking-glass-YYYYMMDD-HHMMSS.log
        let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let log_file = log_dir.join(format!("looking-glass-{}.log", timestamp));

        // Create non-blocking file appender
        let file_appender = tracing_appender::rolling::never(log_dir, format!("looking-glass-{}.log", timestamp));
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        // Determine log level
        let level = if debug {
            Level::DEBUG
        } else if verbose {
            Level::INFO
        } else {
            Level::WARN
        };

        // Create environment filter
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(level.to_string()))
            .add_directive("looking_glass=trace".parse().unwrap());

        // Create JSON formatting layer for file output
        let json_layer = Layer::new()
            .json()
            .with_current_span(false)
            .with_span_list(true)
            .with_file(true)
            .with_line_number(true)
            .with_thread_ids(true)
            .with_thread_names(false)
            .with_target(true)
            .with_level(true)
            .with_writer(non_blocking);

        // Also add a compact stdout layer for immediate feedback (optional, minimal)
        let stdout_layer = Layer::new()
            .with_writer(std::io::stdout)
            .with_ansi(false)
            .compact()
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_file(false)
            .with_line_number(false)
            .with_target(false)
            .with_level(true);

        // Initialize the subscriber
        tracing_subscriber::registry()
            .with(env_filter)
            .with(json_layer)
            .with(stdout_layer)
            .try_init()
            .context("Failed to initialize logging subscriber")?;

        tracing::info!(
            log_file = %log_file.display(),
            level = %level,
            "Logging initialized"
        );

        Ok(LoggingGuard { _guard: guard })
    }
}

/// Get the log directory following XDG Base Directory Specification.
fn get_log_directory() -> Result<PathBuf> {
    let base_dir = if let Ok(xdg_data_home) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(xdg_data_home)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".local/share")
    } else {
        anyhow::bail!("Unable to determine home directory for log files");
    };

    Ok(base_dir.join("LookingGlass/logs"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_log_directory_from_home() {
        // This test just verifies the function doesn't panic
        let _ = get_log_directory();
    }
}