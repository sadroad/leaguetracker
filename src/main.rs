use axum::{debug_handler, http::StatusCode, response::IntoResponse, Router};
use axum::Json;
use dotenvy::dotenv;
use riven::RiotApi;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{env, net::SocketAddr, sync::Arc};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use axum::routing::get;
use axum::extract::State;
use serde_json::json;
use serde::{Serialize, Deserialize};
use ts_rs::TS;

mod gamewatcher;
use gamewatcher::start_game_watcher;

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
    let riot_api = Arc::new(RiotApi::new(&riot_api_key));

    let state = AppState { conn: pool.clone()};

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
    axum::Server::bind(&addr) .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[derive(sqlx::FromRow, Serialize, Deserialize, TS)]
#[ts(export)]
struct Game {
    id: i32,
    name: String,
    kills: i32,
    deaths: i32,
    assists: i32,
    primary_rune: i32,
    secondary_rune: i32,
    summoner_spell_1: i32,
    summoner_spell_2: i32,
    #[sqlx(rename="champion_id")]
    champion: i32,
    champion_name: String,
    game_duration: i64,
    game_completion_time: i64,
    win: bool,
    item_0: i32,
    item_1: i32,
    item_2: i32,
    item_3: i32,
    item_4: i32,
    item_5: i32,
    item_6: i32,
}

#[debug_handler]
async fn get_games_handler(state: State<AppState>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let mut games = sqlx::query_as::<_, Game>("SELECT * FROM games")
        .fetch_all(conn.as_ref())
        .await
        .unwrap();
    games.reverse();
    Json(json!({"games": games}))
}

#[debug_handler]
async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 - Not Found")
}
