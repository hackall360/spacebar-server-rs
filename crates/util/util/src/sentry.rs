use config::SentryConfiguration;
use sentry::ClientInitGuard;

/// Sentry integration helpers.
pub struct Sentry;

impl Sentry {
    /// Initialise Sentry based on configuration.
    /// Returns a guard that should be kept alive for the lifetime of the application.
    pub fn init(cfg: &SentryConfiguration) -> Option<ClientInitGuard> {
        if !cfg.enabled {
            return None;
        }

        let mut opts = sentry::ClientOptions::new();
        opts.traces_sample_rate = cfg.trace_sample_rate;
        if let Some(env) = &cfg.environment {
            opts.environment = Some(env.clone().into());
        }

        Some(sentry::init((cfg.endpoint.as_str(), opts)))
    }
}
