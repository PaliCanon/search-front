//! The site's SSR executable. Configuration failures log bounded variable
//! names, never values. The deployed release identity is shared with hydration.

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use search_front::telemetry::server;
    server::init();
    if let Err(stage) = serve().await {
        server::failed(stage);
        std::process::exit(2);
    }
    server::stopped();
}

#[cfg(feature = "ssr")]
async fn serve() -> Result<(), &'static str> {
    use leptos::prelude::get_configuration;
    use search_front::{
        boot::{BootError, Config},
        origin::Origin,
        router::router,
        telemetry::server,
    };
    use std::sync::Arc;
    let started = std::time::Instant::now();
    let config =
        Config::from_lookup(|name| std::env::var(name).ok()).map_err(BootError::variable)?;
    let origin =
        Origin::from_lookup(|name| std::env::var(name).ok()).map_err(BootError::variable)?;
    let mock = matches!(&config, Config::Mock);
    let checked = matches!(&origin, Origin::Checked(_));
    let source = Arc::new(config.into_source());
    let conf = get_configuration(None).map_err(|_| "leptos_config")?;
    let options = conf.leptos_options;
    let addr = options.site_addr;
    let app = router(options, source, origin);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|_| "listen")?;
    server::started(checked, mock, started.elapsed());
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(server::shutdown())
        .await
        .map_err(|_| "serve")
}

#[cfg(not(feature = "ssr"))]
fn main() {}
