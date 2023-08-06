CREATE TABLE IF NOT EXISTS games (
    id SERIAL PRIMARY KEY ,
    name TEXT NOT NULL,
    kills INT NOT NULL,
    deaths INT NOT NULL,
    assists INT NOT NULL,
    primary_rune INT NOT NULL,
    secondary_rune INT NOT NULL,
    summoner_spell_1 INT NOT NULL,
    summoner_spell_2 INT NOT NULL,
    champion_id INT NOT NULL,
    champion_name TEXT NOT NULL,
    game_duration BIGINT NOT NULL,
    game_completion_time BIGINT NOT NULL,
    win BOOL NOT NULL,
    match_id TEXT NOT NULL 
)
