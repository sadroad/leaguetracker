use axum::{
    debug_handler, extract::State, http::StatusCode, response::IntoResponse, routing::get, Json,
    Router,
};
use libsql_client::Statement;
use libsql_client::Value;
use riven::RiotApi;
use serde::{Deserialize, Serialize};
use serde_json::json;
use libsql_client::client::Client;
use std::{
    env,
    net::{IpAddr, Ipv6Addr, SocketAddr},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use ts_rs::TS;

mod gamewatcher;
use gamewatcher::start_game_watcher;

#[derive(Clone)]
pub struct AppState {
    client: Arc<Client>,
    new_game: Arc<AtomicBool>,
    games: Arc<Mutex<Vec<Game>>>,
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
    if cfg!(debug_assertions) {
        dotenvy::dotenv()?;
    }
    let client = Arc::new(Client::from_env().await?);
    let riot_api_key = env::var("RGAPI_KEY")?;
    let riot_api = Arc::new(RiotApi::new(&riot_api_key));

    let state = AppState {
        client,
        new_game: Arc::new(AtomicBool::new(true)),
        games: Arc::new(Mutex::new(Vec::new())),
    };

    let gamewatcher_state = state.clone();
    tokio::spawn(async move {
        _ = start_game_watcher(riot_api, gamewatcher_state).await;
    });

    let app = Router::new()
        .route("/api/get_games", get(get_games_handler))
        .fallback(handler_404)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = env::var("PORT").expect("Missing Port Number");
    let port = port.parse::<u16>().expect("Invalid Port Number");
    let addr = &SocketAddr::new(IpAddr::from(Ipv6Addr::UNSPECIFIED), port);
    tracing::debug!("Listening on {}", addr);
    axum::Server::bind(addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[derive(TS, Serialize, Deserialize, Clone, Debug)]
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
    champion_id: i32,
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
    let games = if state.new_game.load(Ordering::Relaxed) {
        let client = state.client.clone();
        let mut games: Vec<Game> = client.execute(Statement::new("select (id, name, kills, deaths, assists, primary_rune, secondary_rune, summoner_spell_1, summoner_spell_2, champion_id, champion_name, game_duration, game_completion_time, win, item_0, item_1, item_2, item_3, item_4, item_5, item_6) from games")).await.unwrap().rows.iter().map(|row| {
            let name = row.try_get::<&str>(1).unwrap().to_string();
            let champion_name = row.try_get::<&str>(10).unwrap().to_string();
            let win = match *row.values.get(13).unwrap() {
                Value::Integer { value } => value == 1,
                _ => false,
            };
            Game {
                id: row.try_get(0).unwrap(),
                name,
                kills: row.try_get(2).unwrap(),
                deaths: row.try_get(3).unwrap(),
                assists: row.try_get(4).unwrap(),
                primary_rune: row.try_get(5).unwrap(),
                secondary_rune: row.try_get(6).unwrap(),
                summoner_spell_1: row.try_get(7).unwrap(),
                summoner_spell_2: row.try_get(8).unwrap(),
                champion_id: row.try_get(9).unwrap(),
                champion_name,
                game_duration: row.try_get(11).unwrap(),
                game_completion_time: row.try_get(12).unwrap(),
                win,
                item_0: row.try_get(14).unwrap(),
                item_1: row.try_get(15).unwrap(),
                item_2: row.try_get(16).unwrap(),
                item_3: row.try_get(17).unwrap(),
                item_4: row.try_get(18).unwrap(),
                item_5: row.try_get(19).unwrap(),
                item_6: row.try_get(20).unwrap(), }
        }).collect();
        games.reverse();
        *state.games.lock().unwrap() = games.clone();
        state.new_game.store(false, Ordering::Relaxed);
        games
    } else {
        tracing::info!("Returning cached games");
        state.games.lock().unwrap().clone()
    };
    Json(json!({ "games": games }))
}

#[debug_handler]
async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 - Not Found")
}
