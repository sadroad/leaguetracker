import { component$, useComputed$ } from "@builder.io/qwik";
import { Game } from "../../../bindings/Game";
import { NORMALIZATION_MAP } from "../../utilities/normalization";
import { SPELLS } from "../../utilities/spells";
import { SECONDARY_RUNES } from "../../utilities/secondaryrunes";

interface MatchCardProps {
  game: Game;
  primary_rune: string;
  items: string[];
}

const formattedDifferenceDate = (date_milli: any) => {
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

export const MatchCard = component$<MatchCardProps>((props) => {
  const game = props.game;
  const champ_name = game.champion_name.toLowerCase();
  const items = props.items.slice(0, 6);
  const ward = props.items[6];
  const secondsToMinutesAndSeconds = useComputed$(() => {
    const seconds = Number(game.game_duration);
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    if (remainingSeconds < 10) {
      return `${minutes}:0${remainingSeconds}`;
    }
    return `${minutes}:${remainingSeconds}`;
  });
  return (
    <div
      id="flex-wrapper"
      class={`flex ${
        game.win ? "bg-[#85D7FF]" : "bg-[#FF8585]"
      } gap-x-5 rounded-3xl mx-72 flex-nowrap`}
    >
      <div id="gameinfo" class="ml-5">
        <img
          src={`https://raw.communitydragon.org/latest/game/assets/characters/${champ_name}/hud/${NORMALIZATION_MAP[champ_name]}`}
        />
        <div id="gameduration" class="text-center text-2xl font-semibold">
          {secondsToMinutesAndSeconds.value}
        </div>
        <div id="timecompleted" class="text-center">
          {formattedDifferenceDate(game.game_completion_time)}
        </div>
      </div>
      <div id="playerscore" class="flex flex-col gap-y-4 justify-center">
        <div id="playername" class="text-center text-5xl font-semibold">
          {game.name}
        </div>
        <div id="scoreline" class="text-center text-[2rem] pb-6">
          {game.kills}/{game.deaths}/{game.assists}
        </div>
      </div>
      <div class="flex-grow" />
      <div id="runesAndSums" class="flex flex-col justify-center gap-y-3">
        <div id="runes" class="flex items-center gap-x-3 justify-center">
          <img
            class="w-16 h-16 rounded-full bg-black shrink-0"
            src={`https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/perk-images/styles/${props.primary_rune}`}
            loading="lazy"
          />
          <img
            class="w-7 h-7 rounded-full bg-black shrink-0"
            src={`https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/perk-images/styles/${
              SECONDARY_RUNES[game.secondary_rune]
            }`}
            loading="lazy"
          />
        </div>
        <div id="sums" class="flex">
          <img
            src={`https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/${
              SPELLS[game.summoner_spell_1]
            }`}
            loading="lazy"
          />
          <img
            src={`https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/${
              SPELLS[game.summoner_spell_2]
            }`}
            loading="lazy"
          />
        </div>
      </div>
      <div id="items" class="grid grid-cols-3 grid-rows-2 pt-4">
        {items.map((item) => {
          if (item === undefined) {
            return <div></div>;
          } else {
            return (
              <img
                src={`https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/items/icons2d/${item}`}
                loading="lazy"
              />
            );
          }
        })}
      </div>
      <div id="ward" class="inline-block pt-[3.6rem] mr-3">
        <img
          src={`https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/items/icons2d/${ward}`}
          loading="lazy"
        />
      </div>
    </div>
  );
});
