use axum::{debug_handler, http::StatusCode, response::IntoResponse, Router};
use dotenvy::dotenv;
use riven::RiotApi;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{env, net::SocketAddr, sync::Arc};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use axum::routing::get;
use axum::extract::State;

mod gamewatcher;
use gamewatcher::start_game_watcher;

const CDN_BASE_URL: &str =
    "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/";

#[derive(Clone)]
struct AppState {
    conn: Arc<Pool<Postgres>>,
    riot: Arc<RiotApi>,
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
    let riot_api = Arc::new(RiotApi::new(&riot_api_key));

    let state = AppState { conn: pool.clone(), riot: riot_api.clone() };

    tokio::spawn(async move {
        _ = start_game_watcher(riot_api, &pool).await;
    });

    let app = Router::new()
        .route("/api/get_games", get(get_games_handler))
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
async fn get_games_handler(state: State<AppState>) -> impl IntoResponse {
    "hi"
}

#[debug_handler]
async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 - Not Found")
}
