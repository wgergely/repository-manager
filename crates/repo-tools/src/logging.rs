use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// Initialize a tracing subscriber with default configuration.
///
/// This sets up a subscriber that prints formatted logs to stdout.
/// It uses the `RUST_LOG` environment variable to determine the log level,
/// defaulting to "info" if not set.
pub fn init() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_level(true)
        .with_file(true)
        .with_line_number(true)
        .compact(); // Use compact format for cleaner output

    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .try_init()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::{error, info, warn};

    #[test]
    fn test_logging_init() {
        // We can only init once per process, so we use a check
        let _ = init();

        info!("This is an info message");
        warn!("This is a warning message");
        error!("This is an error message");
    }
}
