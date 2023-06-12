<script>
    import { NORMALIZATION_MAP} from "../utility/normalization";
    import {SPELLS } from "../utility/spells";
    import { SECONDARY_RUNES } from "../utility/secondaryrunes";
    export let game;
    console.log(game);
    const champ_name = game.champion_name.toLowerCase();
    const formattedDifferenceDate = (date_milli) => {
        const date = date_milli;
        const now = Date.now();
        const diff = now - date;
        const diffSeconds = Math.floor(diff / 1000);
        const diffMinutes = Math.floor(diffSeconds / 60);
        const diffHours = Math.floor(diffMinutes / 60);
        const diffDays = Math.floor(diffHours / 24);
        const diffMonths = Math.floor(diffDays / 30);
        const diffYears = Math.floor(diffMonths / 12);
        if (diffYears > 0) {
            return `${diffYears} years ago`;
        } else if (diffMonths > 0) {
            return `${diffMonths} months ago`;
        } else if (diffDays > 0) {
            return `${diffDays} days ago`;
        } else if (diffHours > 0) {
            return `${diffHours} hours ago`;
        } else if (diffMinutes > 0) {
            return `${diffMinutes} minutes ago`;
        } else if (diffSeconds > 0) {
            return `${diffSeconds} seconds ago`;
        } else {
            return `just now`;
        }
    };
    const getSummonerImage = (summoner_id) => {
        return SPELLS[summoner_id];
    };
    const getSecondaryRuneImage = (rune_id) => {
        return SECONDARY_RUNES[rune_id];
    };
    getSummonerImage(game.summoner_spell_1);
</script>

<div id="flex-wrapper">
    <div id="gameinfo">
        <img src="https://raw.communitydragon.org/latest/game/assets/characters/{champ_name}/hud/{NORMALIZATION_MAP[champ_name]}" />
        <div id="gameduration">{game.game_duration}</div>
        <div id="timecompleted">{formattedDifferenceDate(game.game_completion_time)}</div>
    </div>
    <div id="playerscore">
        <div id="playername">{game.name}</div>
        <div id="scoreline">{game.kills}/{game.deaths}/{game.assists}</div>
    </div>
    <div id="runes">
        <div id="runes">
            <img src="https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/perk-images/styles/{getSecondaryRuneImage(game.secondary_rune)}" />
        </div>
        <div id="sums">
            <img src="https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/{getSummonerImage(game.summoner_spell_1)}" />
            <img src="https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/{getSummonerImage(game.summoner_spell_2)}" />
        </div>
    </div>
    <div id="items"></div>
</div>
