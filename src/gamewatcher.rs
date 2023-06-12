use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use phf::phf_map;
use riven::{
    consts::{Champion, PlatformRoute, Queue, RegionalRoute},
    models::summoner_v4::Summoner,
    RiotApi,
};
use tracing::info;
use sqlx::{Pool, Postgres};
use std::{
    collections::HashMap,
    path::Path,
    sync::Arc,
    time::{self, Duration},
};
use tokio::{
    fs::File,
    io::AsyncReadExt,
    sync::{mpsc, Mutex},
    time::sleep,
};

static SUMMONER_SPELLS: phf::Map<i32, &'static str> = phf_map! {
    1_i32 => "Cleanse",
    3_i32 => "Exhaust",
    4_i32 => "Flash",
    6_i32 => "Ghost",
    7_i32 => "Heal",
    11_i32 => "Smite",
    12_i32 => "Teleport",
    13_i32 => "Clarity",
    14_i32 => "Ignite",
    21_i32 => "Barrier",
};

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
    champion_name: String,
    game_completion: i64,
    game_duration: i64,
    win: bool,
    match_id: String,
}

pub async fn start_game_watcher(
    riot_api: Arc<RiotApi>,
    db_pool: &Pool<Postgres>,
) -> anyhow::Result<()> {
    let accounts = Arc::new(Mutex::new(HashMap::new()));

    load_players(&accounts.clone(), &riot_api.clone()).await;

    let file_accounts = accounts.clone();
    let riot_api_file = riot_api.clone();

    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut watcher = RecommendedWatcher::new(
        move |res| match res {
            Ok(event) => {
                tx.send(event).unwrap();
            }
            Err(e) => info!("watch error: {:?}", e),
        },
        Config::default(),
    )
    .unwrap();

    watcher
        .watch(Path::new("."), RecursiveMode::Recursive)
        .unwrap();

    tokio::spawn(async move {
        while let Some(res) = rx.recv().await {
            let file_name = res.paths[0].to_str().unwrap().split("./").last().unwrap();
            if file_name != "players" {
                continue;
            }
            load_players(&file_accounts, &riot_api_file).await;
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
        for summoner in accounts.lock().await.values() {
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
                info!("{} played a match in the last minute", summoner.name);
                for match_id in match_list.iter() {
                    let game = match riot_api
                        .match_v5()
                        .get_match(RegionalRoute::AMERICAS, match_id)
                        .await?
                    {
                        Some(game) => game,
                        None => {
                            info!("Unable to find match_id: {}", match_id);
                            continue;
                        }
                    };
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
                        primary_rune: player_particpant_data.perks.styles[0].selections[0].perk,
                        secondary_rune: player_particpant_data.perks.styles[1].style,
                        summoner_spell_1: player_particpant_data.summoner1_id,
                        summoner_spell_2: player_particpant_data.summoner2_id,
                        champion_id: player_particpant_data.champion().unwrap().0 as i32,
                        champion_name: Champion(player_particpant_data.champion().unwrap().0)
                            .identifier()
                            .unwrap()
                            .to_string(),
                        game_duration: game.info.game_duration,
                        game_completion: game.info.game_end_timestamp.unwrap(),
                        win: player_particpant_data.win,
                        match_id: match_id.to_string(),
                    };

                    sqlx::query("INSERT INTO games (name, kills, deaths, assists, primary_rune, secondary_rune, summoner_spell_1, summoner_spell_2, champion_id, champion_name, game_duration, game_completion_time, win, match_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)")
                        .bind(&player_stats.name)
                        .bind(player_stats.kills)
                        .bind(player_stats.deaths)
                        .bind(player_stats.assists)
                        .bind(player_stats.primary_rune)
                        .bind(player_stats.secondary_rune)
                        .bind(player_stats.summoner_spell_1)
                        .bind(player_stats.summoner_spell_2)
                        .bind(player_stats.champion_id)
                        .bind(&player_stats.champion_name)
                        .bind(player_stats.game_duration)
                        .bind(player_stats.game_completion)
                        .bind(player_stats.win)
                        .bind(&player_stats.match_id)
                        .execute(db_pool)
                        .await?;

                    info!("Player: {}", player_stats.name);
                    info!(
                        "KDA: {}/{}/{}",
                        player_stats.kills, player_stats.deaths, player_stats.assists
                    );
                    info!("Primary Tree: {}", player_stats.primary_rune);
                    info!("Secondary Tree: {}", player_stats.secondary_rune);
                    info!(
                        "Summoner Spell 1: {}",
                        SUMMONER_SPELLS.get(&player_stats.summoner_spell_1).unwrap()
                    );
                    info!(
                        "Summoner Spell 2: {}",
                        SUMMONER_SPELLS.get(&player_stats.summoner_spell_2).unwrap()
                    );
                    info!("Champion: {}", player_stats.champion_name);
                    info!("Game Duration: {}", player_stats.game_duration);
                    info!("Game Completion: {}", player_stats.game_completion);
                    info!("Win: {}", player_stats.win);
                }
            };
        }
    }
    Ok(())
}

async fn load_players(
    global_accounts: &Arc<Mutex<HashMap<String, Summoner>>>,
    riot_api: &Arc<RiotApi>,
) {
    let last_accounts_in_file = {
        global_accounts
            .lock()
            .await
            .keys()
            .cloned()
            .collect::<Vec<String>>()
    };
    let mut buffer = Vec::new();
    {
        let mut player_file = File::open("players").await.unwrap();
        player_file.read_to_end(&mut buffer).await.unwrap();
    }
    let mut current_accounts_in_file = String::from_utf8(buffer)
        .unwrap()
        .split('\n')
        .map(|x| x.to_string())
        .filter(|x| !x.is_empty())
        .collect::<Vec<String>>();

    if current_accounts_in_file.len() < last_accounts_in_file.len() {
        let mut accounts = global_accounts.lock().await;
        last_accounts_in_file
            .iter()
            .filter(|x| !current_accounts_in_file.contains(x))
            .for_each(|x| {
                accounts.remove(x);
            });
        info!("Done removing accounts");
    } else {
        current_accounts_in_file.retain(|x| !last_accounts_in_file.contains(x));
        for account in current_accounts_in_file.iter() {
            let summoner = match riot_api
                .summoner_v4()
                .get_by_summoner_name(PlatformRoute::NA1, account)
                .await
                .unwrap()
            {
                Some(x) => x,
                None => {
                    info!("Could not find summoner {}", account);
                    continue;
                }
            };

            global_accounts
                .lock()
                .await
                .insert(account.clone(), summoner);
        }
        if !current_accounts_in_file.is_empty() || last_accounts_in_file.is_empty() {
            info!("Done loading accounts");
        }
    }
}
