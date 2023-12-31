use libsql_client::{args, Statement};
use md5::{Digest, Md5};
use riven::{
    consts::{Champion, PlatformRoute, Queue, RegionalRoute},
    models::summoner_v4::Summoner,
    RiotApi,
};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{atomic::Ordering, Arc},
    time::{self, Duration},
};
use tokio::{
    fs::File,
    io::AsyncReadExt,
    sync::{mpsc, Mutex},
    time::sleep,
};
use tracing::info;
use uuid::Builder;

use crate::AppState;

#[derive(Debug, Clone, Serialize)]
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
    item_0: i32,
    item_1: i32,
    item_2: i32,
    item_3: i32,
    item_4: i32,
    item_5: i32,
    item_6: i32,
}

pub async fn start_game_watcher(riot_api: Arc<RiotApi>, state: AppState) -> anyhow::Result<()> {
    let client = state.client.clone();
    let accounts = Arc::new(Mutex::new(HashMap::new()));

    load_players(&accounts.clone(), &riot_api.clone()).await;

    //let file_accounts = accounts.clone();
    //let riot_api_file = riot_api.clone();

    //let (tx, mut rx) = mpsc::unbounded_channel();
    //let mut watcher = RecommendedWatcher::new(
    //    move |res| match res {
    //        Ok(event) => {
    //            tx.send(event).unwrap();
    //        }
    //        Err(e) => info!("watch error: {:?}", e),
    //    },
    //    Config::default(),
    //)
    //.unwrap();

    //watcher
    //    .watch(Path::new("."), RecursiveMode::Recursive)
    //    .unwrap();

    //tokio::spawn(async move {
    //    while let Some(res) = rx.recv().await {
    //        let file_name = res.paths[0].to_str().unwrap().split("./").last().unwrap();
    //        if file_name != "players" {
    //            continue;
    //        }
    //        load_players(&file_accounts, &riot_api_file).await;
    //    }
    //});

    let (tx, mut rx) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        loop {
            let current_time = time::SystemTime::now();
            let past = current_time.checked_sub(Duration::from_secs(60)).unwrap();
            let epoch_time = past
                .duration_since(time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            tx.send(epoch_time).unwrap();
            //if let Err(TrySendError(e)) = tx.try_send(epoch_time).await {
            //    dbg!(e);
            //}
            sleep(Duration::from_secs(60)).await;
        }
    });

    loop {
        while let Some(epoch_time) = rx.recv().await {
            println!("Got a time");
            for summoner in accounts.lock().await.values() {
                println!("Trying {}", summoner.name);
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
                        println!("Found the game");
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
                            item_0: player_particpant_data.item0,
                            item_1: player_particpant_data.item1,
                            item_2: player_particpant_data.item2,
                            item_3: player_particpant_data.item3,
                            item_4: player_particpant_data.item4,
                            item_5: player_particpant_data.item5,
                            item_6: player_particpant_data.item6,
                        };

                        let mut hasher = Md5::new();
                        hasher.update(
                            (format!("{}{}", player_stats.match_id, player_stats.name)).as_bytes(),
                        );
                        let md5_hash = hasher.finalize();
                        let uuid = Builder::from_md5_bytes(md5_hash.into()).into_uuid();

                        info!("Inserting into the DB");
                        let transaction = client.transaction().await.unwrap();
                        let rs = transaction
                            .execute(Statement::with_args(
                                "SELECT md5sum FROM games WHERE md5sum = ?",
                                args!(uuid.to_string()),
                            ))
                            .await
                            .unwrap();
                        if !rs.rows.is_empty() {
                            info!("Game already stored, skipping");
                            transaction.rollback().await.unwrap();
                            continue;
                        }
                        let _ = transaction.execute(Statement::with_args("INSERT INTO games (name, kills, deaths, assists, primary_rune, secondary_rune, summoner_spell_1, summoner_spell_2, champion_id, champion_name, game_duration, game_completion_time, win, match_id, item_0, item_1, item_2, item_3, item_4, item_5, item_6, md5sum) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
                        args!(
                            player_stats.name.clone(),
                            player_stats.kills,
                            player_stats.deaths,
                            player_stats.assists,
                            player_stats.primary_rune,
                            player_stats.secondary_rune,
                            player_stats.summoner_spell_1,
                            player_stats.summoner_spell_2,
                            player_stats.champion_id,
                            player_stats.champion_name.clone(),
                            player_stats.game_duration,
                            player_stats.game_completion,
                            i32::from(player_stats.win),
                            player_stats.match_id.clone(),
                            player_stats.item_0,
                            player_stats.item_1,
                            player_stats.item_2,
                            player_stats.item_3,
                            player_stats.item_4,
                            player_stats.item_5,
                            player_stats.item_6,
                            uuid.to_string(),
                        ))).await.unwrap();
                        transaction.commit().await.unwrap();
                        state.new_game.store(true, Ordering::Relaxed);
                        info!("{}", serde_json::to_string_pretty(&player_stats).unwrap());
                    }
                };
            }
            println!("Starting to await new time");
        }
        println!("Broke out of waiting, should loop");
    }
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
