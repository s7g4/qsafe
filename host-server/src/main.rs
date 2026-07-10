use qsafe_backend::app::{build_router, AppState};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber for structured JSON logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    // Initialize Prometheus exporter
    let prometheus_handle = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
        .map_err(|e| format!("Failed to install Prometheus recorder: {}", e))?;

    // Load and validate environment configuration
    let config = qsafe_backend::config::Config::load()?;
    let state = AppState::build(&config).await?;
    let app = build_router(state, &config.cors_origin, Some(prometheus_handle));

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));

    // tower_governor's rate limiter keys requests by peer IP, which it reads
    // from the `ConnectInfo<SocketAddr>` request extension. That extension is
    // only populated when the service is built with
    // `into_make_service_with_connect_info`; a plain `into_make_service()`
    // (or handing the bare `Router` to `axum::serve`) leaves it unset and
    // every rate-limited route (all of `/api/auth/*`) 500s with
    // "Unable To Extract Key!" on every real request.
    if let (Some(cert), Some(key)) = (config.tls_cert_path.clone(), config.tls_key_path.clone()) {
        println!("Q-Safe Backend Server running with TLS on {}", addr);
        let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(cert, key)
            .await
            .expect("Failed to load TLS certificates");

        axum_server::bind_rustls(addr, tls_config)
            .handle(shutdown_handle())
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .expect("Server failed");
    } else {
        println!("Q-Safe Backend Server running on {}", addr);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    }

    Ok(())
}

fn shutdown_handle() -> axum_server::Handle<std::net::SocketAddr> {
    let handle = axum_server::Handle::new();
    let spawn_handle = handle.clone();
    tokio::spawn(async move {
        shutdown_signal().await;
        spawn_handle.graceful_shutdown(Some(std::time::Duration::from_secs(30)));
    });
    handle
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
    tracing::info!("Shutdown signal received, starting graceful shutdown...");
}
