use dotenvy::dotenv;
use riven::consts::PlatformRoute;
use riven::consts::Queue;
use riven::consts::RegionalRoute;
use riven::RiotApi;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::{self, Duration};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;
use tokio::time::sleep;

#[derive(Debug, Clone)]
struct PlayerStats {
    name: String,
    kills: i32,
    deaths: i32,
    assists: i32,
    primary_rune: i32,
    secondary_rune: i32,
    summoner_spell_1: i32,
    summoner_spell_2: i32,
    champion_id: i32,
    game_completion: i64,
    game_duartion: i64,
    win: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv()?;
    //let connection_string = env::var("DATABASE_URL")?;
    //let pool = PgPoolOptions::new()
    //    .max_connections(5)
    //    .connect(&connection_string).await?;
    //let row: (i64,) = sqlx::query_as("SELECT id2 FROM test")
    //    .fetch_one(&pool)
    //    .await?;
    //dbg!(&row);

    let riot_api_key = env::var("RGAPI_KEY")?;
    let riot_api = Arc::new(RiotApi::new(riot_api_key));

    let accounts = Arc::new(Mutex::new(Vec::new()));

    let file_accounts = accounts.clone();
    let riot_api_file = riot_api.clone();

    tokio::spawn(async move {
        let mut last_checksum = 0_u32;
        let mut last_accounts_in_file = Vec::new();
        loop {
            let mut player_file = File::open("players").await.unwrap();
            let mut buffer = Vec::new();
            player_file.read_to_end(&mut buffer).await.unwrap();
            let current_checksum = crc32fast::hash(&buffer);
            if last_checksum != current_checksum {
                let current_accounts_in_file= String::from_utf8(buffer)
                    .unwrap()
                    .split('\n')
                    .map(|x| x.to_string())
                    .filter(|x| !x.is_empty())
                    .collect::<Vec<String>>();
                for account in current_accounts_in_file.iter() {
                    if !last_accounts_in_file.contains(account) {
                        println!("{} added to accounts", account);
                    }
                }
                last_accounts_in_file = current_accounts_in_file.clone();
                last_checksum = current_checksum;

                let mut accounts = Vec::new();
                for account in current_accounts_in_file.iter() {
                    let summoner = riot_api_file
                        .summoner_v4()
                        .get_by_summoner_name(PlatformRoute::NA1, account)
                        .await
                        .unwrap()
                        .expect("No summoner found");
                    accounts.push(summoner);
                }
                *file_accounts.lock().unwrap() = accounts;

                println!("Done loading accounts");
            }
        }
    });

    let (tx, mut rx) = mpsc::channel(100);

    tokio::spawn(async move {
        let mut current_time = time::SystemTime::now();
        loop {
            sleep(Duration::from_secs(60)).await;
            let epoch_time = current_time
                .duration_since(time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            current_time = time::SystemTime::now();
            tx.send(epoch_time).await.unwrap();
        }
    });

    while let Some(epoch_time) = rx.recv().await {
        for summoner in accounts.lock().unwrap().iter() {
            if let Ok(match_list) = riot_api
                .match_v5()
                .get_match_ids_by_puuid(
                    RegionalRoute::AMERICAS,
                    &summoner.puuid,
                    Some(20),
                    None,
                    Some(Queue::SUMMONERS_RIFT_5V5_RANKED_SOLO),
                    Some(epoch_time.try_into().unwrap()),
                    None,
                    None,
                )
                .await
            {
                if match_list.is_empty() {
                    continue;
                }
                println!("{} played a match in the last minute", summoner.name);
                for match_id in match_list.iter() {
                    let game = riot_api
                        .match_v5()
                        .get_match(RegionalRoute::AMERICAS, match_id)
                        .await?
                        .unwrap();
                    let player_particpant_data = game
                        .info
                        .participants
                        .iter()
                        .find(|x| x.puuid == summoner.puuid)
                        .unwrap();
                    let player_stats = PlayerStats {
                        name: player_particpant_data.summoner_name.clone(),
                        kills: player_particpant_data.kills,
                        deaths: player_particpant_data.deaths,
                        assists: player_particpant_data.assists,
                        primary_rune: player_particpant_data.perks.styles[0].style,
                        secondary_rune: player_particpant_data.perks.styles[1].style,
                        summoner_spell_1: player_particpant_data.summoner1_id,
                        summoner_spell_2: player_particpant_data.summoner2_id,
                        champion_id: player_particpant_data.champion().unwrap().0 as i32,
                        game_duartion: game.info.game_duration,
                        game_completion: game.info.game_end_timestamp.unwrap(),
                        win: player_particpant_data.win,
                    };
                    dbg!(player_stats);
                }
            };
        }
    }

    const CDN_BASE_URL: &str =
        "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/";

    Ok(())
}
