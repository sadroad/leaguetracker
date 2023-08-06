use libsql_client::{args, Client, Config, Statement};
use sqlx::{postgres::PgPoolOptions, types::Uuid};

#[derive(sqlx::FromRow, Debug)]
struct Entry {
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
    match_id: String,
    item_0: i32,
    item_1: i32,
    item_2: i32,
    item_3: i32,
    item_4: i32,
    item_5: i32,
    item_6: i32,
    md5sum: Uuid,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgresql://postgres:EVbmTUkm7L7ZSsuWBQTU@containers-us-west-65.railway.app:7498/railway")
        .await?;
    let entries = sqlx::query_as::<_, Entry>("SELECT * FROM games")
        .fetch_all(&pool)
        .await?;
    let config = Config {
        url: "libsql://league-tracker-sadroad.turso.io".try_into()?,
        auth_token: Some(String::from("eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE2OTEyODA0NDAsImlkIjoiZTJjNGZiYTQtMzNkZC0xMWVlLWI3OGUtMzY5OGMyNDkwNzZjIn0.nRgwwr9NvtmJnSqf-MZzRDmLVLjCIfnDJsS-rDeTx3i0C7aPcL9or5Nl9Ohb_7SoEOhl10Xfstwqn_YnBbiVCQ"))
    };
    let client = Client::from_config(config).await?;
    for entry in entries {
        let transaction = client.transaction().await?;
        let rs = transaction
            .execute(Statement::with_args(
                "select md5sum from games where md5sum = ? limit 1",
                args!(entry.md5sum.to_string()),
            ))
            .await?;
        if rs.rows.len() != 0 {
            println!("Skipping {}", entry.md5sum);
            transaction.rollback().await?;
            continue;
        }
        let _ = transaction.execute(Statement::with_args(
                "insert into games (id, name, kills, deaths, assists, primary_rune, secondary_rune, summoner_spell_1, summoner_spell_2, champion_id, champion_name, game_duration, game_completion_time, win, match_id, item_0, item_1, item_2, item_3, item_4, item_5, item_6, md5sum) values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                args!(entry.id, entry.name, entry.kills, entry.deaths, entry.assists, entry.primary_rune, entry.secondary_rune, entry.summoner_spell_1, entry.summoner_spell_2, entry.champion_id, entry.champion_name, entry.game_duration, entry.game_completion_time, entry.win.to_string(), entry.match_id, entry.item_0, entry.item_1, entry.item_2, entry.item_3, entry.item_4, entry.item_5, entry.item_6, entry.md5sum.to_string())
                )).await?;
        transaction.commit().await?;
    }
    Ok(())
}
