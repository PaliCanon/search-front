//! This service's server: renders the page, serves the WASM bundle and
//! the assets. Run with `cargo leptos watch` from the repository root
//! (README.md: the task's own port, 3101). `SEARCH_API_URL` and
//! `SEARCH_API_TOKEN` (both, or neither for the mock) are read once at
//! boot; a bad pair stops the service before it listens.

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use std::sync::Arc;

    use leptos::prelude::get_configuration;
    use search_front::boot::Config;
    use search_front::origin::Origin;
    use search_front::router::router;

    // The error names a variable, never a value.
    let config = match Config::from_lookup(|name| std::env::var(name).ok()) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("search-front: {err}");
            std::process::exit(2);
        }
    };
    let origin = match Origin::from_lookup(|name| std::env::var(name).ok()) {
        Ok(origin) => origin,
        Err(err) => {
            eprintln!("search-front: {err}");
            std::process::exit(2);
        }
    };
    println!("{}", config.log_line());
    println!("{}", origin.log_line());
    println!(
        "{}",
        search_front::release::ReleaseInfo::server().log_line()
    );
    let source = Arc::new(config.into_source());

    let conf = get_configuration(None).expect("leptos configuration");
    let options = conf.leptos_options;
    let addr = options.site_addr;
    let app = router(options, source, origin);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind this service's address");
    println!("search-front on http://{addr}");
    axum::serve(listener, app.into_make_service())
        .await
        .expect("serve this service");
}

#[cfg(not(feature = "ssr"))]
fn main() {}
