use axum::{debug_handler, http::StatusCode, response::IntoResponse, Router};
use dotenvy::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{env, net::SocketAddr, sync::Arc};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod gamewatcher;
use gamewatcher::start_game_watcher;

const CDN_BASE_URL: &str =
    "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/";

#[derive(Clone)]
struct AppState {
    conn: Arc<Pool<Postgres>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    dotenv()?;
    let connection_string = env::var("DATABASE_URL")?;
    let pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&connection_string)
            .await?,
    );
    let riot_api_key = env::var("RGAPI_KEY")?;

    let state = AppState { conn: pool.clone() };

    tokio::spawn(async move {
        _ = start_game_watcher(&riot_api_key, &pool).await;
    });

    let app = Router::new()
        .nest_service("/", ServeDir::new("public"))
        .fallback(handler_404)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::debug!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[debug_handler]
async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 - Not Found")
}
