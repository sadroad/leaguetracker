use dotenvy::dotenv;
use riven::RiotApi;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions};
use std::env;

#[derive(sqlx::FromRow, Serialize, Deserialize)]
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
    #[sqlx(rename = "champion_id")]
    champion: i32,
    champion_name: String,
    game_duration: i64,
    game_completion_time: i64,
    win: bool,
    match_id: String,
    item_0: i32,
    item_1: i32,
    item_2: i32,
    item_3: i32,
    item_4: i32,
    item_5: i32,
    item_6: i32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv()?;
    let riot_api = env::var("RIOT_API_KEY")?;
    let riot = RiotApi::new(&riot_api);
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&env::var("DATABASE_URL")?)
        .await?;

    let games = sqlx::query_as::<_, Game>("SELECT * FROM games")
        .fetch_all(&pool)
        .await?;
    for game in games {
        if game.item_0 == 0
            && game.item_1 == 0
            && game.item_2 == 0
            && game.item_3 == 0
            && game.item_4 == 0
            && game.item_5 == 0
            && game.item_6 == 0
        {
            let match_game = riot.match_v5().get_match(riven::consts::RegionalRoute::AMERICAS,&game.match_id).await?.unwrap();
            let participant = match_game
                .info
                .participants
                .iter()
                .find(|p| p.summoner_name == game.name)
                .unwrap();
            sqlx::query(
                "UPDATE games SET item_0 = $1, item_1 = $2, item_2 = $3, item_3 = $4, item_4 = $5, item_5 = $6, item_6 = $7 WHERE name = $8 AND match_id = $9",
            )
            .bind(participant.item0)
            .bind(participant.item1)
            .bind(participant.item2)
            .bind(participant.item3)
            .bind(participant.item4)
            .bind(participant.item5)
            .bind(participant.item6)
            .bind(game.name)
            .bind(game.match_id)
            .execute(&pool)
            .await?;
        }
    }
    Ok(())
}
