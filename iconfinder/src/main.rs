use riven::consts::Champion;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;

const CHECK: bool = false;

#[tokio::main]
async fn main() {
    if CHECK {
    let mut file = File::create("map").await.unwrap();
    for champion in Champion::ALL_KNOWN.iter() {
        if champion.name().is_none() {
            continue;
        }
        let champ_name = champion.identifier().unwrap().to_lowercase();
        let resp = reqwest::get(format!(
            "https://raw.communitydragon.org/latest/game/assets/characters/{}/hud/{}_circle.png",
            champ_name, champ_name
        ))
        .await
        .unwrap();
        if resp.status().is_client_error() {
            file.write(format!("\"{}\": \"{}_circle_0.png\",\n", champ_name, champ_name).as_bytes()).await.unwrap();
        } else {
            file.write(format!("\"{}\": \"{}_circle.png\",\n", champ_name, champ_name).as_bytes()).await.unwrap();
        }
    }
    } else {
        let mut file = File::open("map").await.unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).await.unwrap();
        let map: std::collections::HashMap<String, String> = serde_json::from_str(&contents).unwrap();
        for champion in Champion::ALL_KNOWN.iter() {
            if champion.name().is_none() {
                continue;
            }
            let champ_name = champion.identifier().unwrap().to_lowercase();
            let resp = reqwest::get(format!(
                "https://raw.communitydragon.org/latest/game/assets/characters/{}/hud/{}",
                champ_name, map.get(&champ_name).unwrap()
                )).await.unwrap();
            if resp.status().is_client_error() {
                println!("{} is incorrect", champ_name);
            }
        }
    }
}
