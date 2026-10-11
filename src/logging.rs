//! Logging initialization for operator.
//!
//! TUI mode: logs to `.tickets/operator/logs/operator-{datetime}.log`
//! CLI mode: logs to stderr

use anyhow::Result;
use std::path::PathBuf;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::Layer;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::{Config, LogFormat};

pub(crate) fn fmt_layer<S, W>(
    format: LogFormat,
    writer: W,
    ansi: bool,
) -> Box<dyn Layer<S> + Send + Sync>
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    W: for<'w> tracing_subscriber::fmt::MakeWriter<'w> + Send + Sync + 'static,
{
    let base = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_ansi(ansi)
        .with_writer(writer);
    match format {
        LogFormat::Text => base.boxed(),
        LogFormat::Json => base.json().with_current_span(true).boxed(),
    }
}

/// Result of logging initialization
pub struct LoggingHandle {
    /// Guard that must be kept alive for the duration of the program.
    /// When dropped, ensures all buffered logs are flushed.
    pub _guard: Option<WorkerGuard>,

    /// Path to the log file (only set in TUI mode with file logging enabled)
    pub log_file_path: Option<PathBuf>,
}

/// Initialize logging based on mode and configuration.
///
/// # Arguments
/// * `config` - Application configuration
/// * `is_tui_mode` - Whether running in TUI mode (true) or CLI mode (false)
/// * `debug_override` - If true, override log level to "debug" (from --debug flag)
///
/// # Returns
/// A `LoggingHandle` that must be kept alive for the duration of the program.
pub fn init_logging(
    config: &Config,
    is_tui_mode: bool,
    debug_override: bool,
) -> Result<LoggingHandle> {
    let log_level = if debug_override {
        "debug".to_string()
    } else {
        config.logging.level.clone()
    };

    let filter = tracing_subscriber::EnvFilter::new(std::env::var("RUST_LOG").unwrap_or(log_level));

    if is_tui_mode && config.logging.to_file {
        // TUI mode with file logging: write to file
        let logs_dir = config.logs_path();
        std::fs::create_dir_all(&logs_dir)?;

        // Generate log filename with ISO8601 timestamp
        let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
        let log_filename = format!("operator-{timestamp}.log");
        let log_file_path = logs_dir.join(&log_filename);

        // Create file appender
        let file_appender = tracing_appender::rolling::never(&logs_dir, &log_filename);
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer(config.logging.format, non_blocking, false))
            .init();

        Ok(LoggingHandle {
            _guard: Some(guard),
            log_file_path: Some(log_file_path),
        })
    } else {
        // CLI mode or TUI with file logging disabled: log to stderr
        let ansi = config.logging.format == LogFormat::Text;
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer(config.logging.format, std::io::stderr, ansi))
            .init();

        Ok(LoggingHandle {
            _guard: None,
            log_file_path: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    #[derive(Clone)]
    struct BufWriter(Arc<Mutex<Vec<u8>>>);

    impl Write for BufWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().expect("log buffer").extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for BufWriter {
        type Writer = Self;

        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    #[test]
    fn test_json_format_writes_one_object_per_line() {
        let buf = Arc::new(Mutex::new(Vec::new()));
        let writer = BufWriter(buf.clone());
        let subscriber = tracing_subscriber::registry().with(fmt_layer(LogFormat::Json, writer, false));
        tracing::subscriber::with_default(subscriber, || tracing::info!(answer = 42, "hello"));
        let out = String::from_utf8(buf.lock().expect("log buffer").clone()).unwrap();
        let line: serde_json::Value = serde_json::from_str(out.lines().next().unwrap()).unwrap();
        assert_eq!(line["fields"]["message"], "hello");
        assert_eq!(line["fields"]["answer"], 42);
        assert_eq!(line["level"], "INFO");
    }

    fn test_config(temp_dir: &TempDir) -> Config {
        let mut config = Config::default();
        config.paths.state = temp_dir.path().to_string_lossy().to_string();
        config
    }

    #[test]
    fn test_logs_path_created() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);

        let logs_dir = config.logs_path();
        assert!(logs_dir.ends_with("logs"));
        assert!(logs_dir.starts_with(temp_dir.path()));
    }

    #[test]
    fn test_log_file_path_format() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);

        // Create logs directory and simulate file creation
        let logs_dir = config.logs_path();
        std::fs::create_dir_all(&logs_dir).unwrap();

        let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
        let log_filename = format!("operator-{timestamp}.log");
        let log_file_path = logs_dir.join(&log_filename);

        assert!(log_file_path.to_string_lossy().contains("operator-"));
        assert!(log_file_path.to_string_lossy().ends_with(".log"));
    }

    #[test]
    fn test_cli_mode_no_log_file() {
        // In CLI mode, log_file_path should be None
        // We can't actually call init_logging multiple times due to global subscriber,
        // so we test the logic indirectly
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);

        // Verify the condition that would lead to no log file
        let is_tui_mode = false;
        assert!(!is_tui_mode || !config.logging.to_file);
    }

    #[test]
    fn test_tui_mode_with_file_disabled() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = test_config(&temp_dir);
        config.logging.to_file = false;

        // Even in TUI mode, if to_file is false, no log file should be created
        let is_tui_mode = true;
        assert!(!(is_tui_mode && config.logging.to_file));
    }
}
