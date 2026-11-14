//! Operational JSON logs, usable without an OpenTelemetry backend.
//!
//! The site installs a tracing subscriber at its two entry points. The CSR
//! fragment only emits events: its host owns the subscriber. No exporter,
//! network call or persistent browser identifier is created here.

#[cfg(all(feature = "browser", target_arch = "wasm32"))]
mod browser;
#[cfg(feature = "ssr")]
pub mod server;

/// The SSR meta carrying the deployment environment to hydration.
pub const ENV_META: &str = "search-deployment-environment";

/// Identity attached to every Search event, independently of its sink.
#[derive(Clone)]
pub(crate) struct Identity {
    pub(crate) environment: String,
    pub(crate) version: String,
    pub(crate) commit: String,
    pub(crate) runtime: &'static str,
}

/// Emit only explicit, sanitized fields; never pass a user value or a raw
/// error through Debug/Display. The macro also avoids reading browser metas
/// when the host has no subscriber or the event is filtered out.
macro_rules! event {
    ($level:ident, $name:literal $(, $($fields:tt)*)?) => {{
        if tracing::enabled!(target: "search", tracing::Level::$level) {
            let identity = crate::telemetry::identity();
            tracing::event!(target: "search", tracing::Level::$level,
                event.name = $name,
                severity = crate::telemetry::severity(tracing::Level::$level),
                service.namespace = "palicanon",
                service.name = "search",
                service.version = %identity.version,
                service.commit = %identity.commit,
                deployment.environment.name = %identity.environment,
                runtime = identity.runtime,
                $($($fields)*)?
            );
        }
    }};
}
pub(crate) use event;

pub(crate) fn severity(level: tracing::Level) -> &'static str {
    match level {
        tracing::Level::ERROR => "ERROR",
        tracing::Level::WARN => "WARNING",
        tracing::Level::INFO => "INFO",
        tracing::Level::DEBUG | tracing::Level::TRACE => "DEBUG",
    }
}

pub(crate) fn identity() -> Identity {
    #[cfg(feature = "ssr")]
    return server::identity().clone();
    #[cfg(all(feature = "browser", target_arch = "wasm32", not(feature = "ssr")))]
    return browser::identity();
    #[cfg(not(any(feature = "ssr", all(feature = "browser", target_arch = "wasm32"))))]
    Identity {
        environment: "unknown".into(),
        version: "unknown".into(),
        commit: String::new(),
        runtime: "host",
    }
}

/// A panic's code location is useful; its payload may contain user data.
#[cfg(any(feature = "ssr", all(feature = "browser", target_arch = "wasm32")))]
pub(crate) fn panic(info: &std::panic::PanicHookInfo<'_>) {
    let file = info
        .location()
        .map(|location| location.file().rsplit('/').next().unwrap_or("unknown"));
    let line = info.location().map(std::panic::Location::line);
    event!(ERROR, "runtime.panic", code.file = file, code.line = line);
}

/// Installs the site's console sink. A CSR host may opt in, or install its
/// own subscriber instead. An existing subscriber is always preserved.
#[cfg(all(feature = "browser", target_arch = "wasm32"))]
pub fn init_browser() {
    browser::init();
}

/// A bounded outcome summary, shared by SSR and the site's browser transport.
#[cfg(feature = "site")]
pub(crate) fn search_outcome(outcome: &crate::search::SearchOutcome, duration_ms: f64) {
    use crate::search::SearchOutcome;
    match outcome {
        SearchOutcome::Hits(response) => event!(
            INFO,
            "search.completed",
            hits = response.hits.len(),
            duration_ms
        ),
        SearchOutcome::Cold { .. } => event!(
            WARN,
            "search.deferred",
            reason = "engine_starting",
            duration_ms
        ),
        SearchOutcome::RateLimited { .. } => event!(
            WARN,
            "search.deferred",
            reason = "rate_limited",
            duration_ms
        ),
        SearchOutcome::Failed => event!(WARN, "search.failed", reason = "backend", duration_ms),
    }
}

/// Monotonic browser time; native hydrate checks use an ordinary clock.
#[cfg(all(feature = "site", feature = "browser"))]
pub(crate) fn now_ms() -> f64 {
    #[cfg(target_arch = "wasm32")]
    return web_sys::window()
        .and_then(|window| window.performance())
        .map_or(0.0, |clock| clock.now());
    #[cfg(not(target_arch = "wasm32"))]
    {
        static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        START
            .get_or_init(std::time::Instant::now)
            .elapsed()
            .as_secs_f64()
            * 1000.0
    }
}

#[cfg(all(test, feature = "ssr"))]
pub(crate) mod tests {
    use std::io::{self, Write};
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    pub(crate) struct Capture(Arc<Mutex<Vec<u8>>>);

    impl Capture {
        /// Everything written so far.
        pub(crate) fn text(&self) -> String {
            String::from_utf8_lossy(&self.0.lock().unwrap()).into_owned()
        }
    }

    impl Write for Capture {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    pub(crate) fn capture(run: impl FnOnce()) -> Vec<serde_json::Value> {
        use tracing_subscriber::prelude::*;
        let output = Capture::default();
        let writer = output.clone();
        let subscriber = tracing_subscriber::registry().with(server_layer(move || writer.clone()));
        tracing::subscriber::with_default(subscriber, run);
        let bytes = output.0.lock().unwrap();
        String::from_utf8_lossy(&bytes)
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    use super::server::layer as server_layer;

    #[test]
    fn json_logs_have_identity_severity_and_only_the_application_target() {
        let logs = capture(|| {
            super::event!(WARN, "test.failed", error.type = "network");
            tracing::error!(target: "http_client", "secret library error");
            tracing::error!(target: "search_dependency", "secret library error");
            super::event!(DEBUG, "test.debug");
        });
        assert_eq!(logs.len(), 1);
        let log = &logs[0];
        assert_eq!(log["event.name"], "test.failed");
        assert_eq!(log["severity"], "WARNING");
        assert_eq!(log["service.namespace"], "palicanon");
        assert_eq!(log["service.name"], "search");
        assert_eq!(log["runtime"], "server");
        assert!(log["timestamp"].as_str().is_some());
        assert!(log["service.version"].as_str().is_some());
        assert_eq!(log["error.type"], "network");
    }
}
