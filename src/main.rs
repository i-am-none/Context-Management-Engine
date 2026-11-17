use nexus_cme::api;
use nexus_cme::config;
use nexus_cme::db;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nexus_cme=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::Config::from_env();
    if let Err(err) = config.validate() {
        eprintln!("Configuration error: {err}");
        std::process::exit(1);
    }
    let addr = config.socket_addr();

    info!(
        database_url = %config.database_url,
        max_connections = config.max_connections,
        "Initializing SQLite database"
    );

    let db_pool = db::init_db(&config.database_url, config.max_connections).await?;
    info!("Database initialized and migrations applied successfully");

    let app_state = api::AppState { db: db_pool };
    let app = api::app_router(app_state);

    info!(address = %addr, "Starting Nexus CME HTTP service");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Nexus CME listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
