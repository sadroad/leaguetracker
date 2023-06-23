import { component$ } from "@builder.io/qwik";
import { routeLoader$, DocumentHead } from "@builder.io/qwik-city";
import { Game } from "../../../bindings/Game.ts";
import { MatchCard } from "../components/matchcard/matchcard";

interface Games {
  games: Game[];
}

export const getGameData = routeLoader$(async () => {
  const endpoint = "http://localhost:8080/api/get_games";
  const res = await fetch(endpoint);
  const data = await res.json();
  return data as Games;
});

export const getPrimaryRunes = routeLoader$(async () => {
  const res = await fetch(
    "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/perks.json"
  );
  const rune_data = await res.json();
  const rune_map = rune_data.reduce((acc, rune) => {
    const rune_path = rune.iconPath
      .slice(rune.iconPath.indexOf("Styles") + 7)
      .toLowerCase();
    acc[rune.id] = rune_path;
    return acc;
  }, {});
  return rune_map;
});

export const getItems = routeLoader$(async () => {
  const res = await fetch(
    "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/items.json"
  );
  const item_data = await res.json();
  const item_map = item_data.reduce((acc, item) => {
    acc[item.id] = item.iconPath
      .slice(item.iconPath.indexOf("Icons2D") + 8)
      .toLowerCase();
    return acc;
  }, {});
  return item_map;
});

export default component$(() => {
  const game_data = getGameData();
  const games = game_data.value.games.sort((a, b) => {
    if (a.id === b.id) {
      return 0;
    }
    return a.id < b.id ? 1 : -1;
  });
  const primary_runes = getPrimaryRunes();
  const get_items = getItems();
  const vecItems = (game: Game) => {
    const items = [];
    for (let i = 0; i < 7; i++) {
      const item = game[`item_${i}`];
      items.push(get_items.value[item]);
    }
    return items;
  };

  return (
    <>
      <div id="navbar">
        <h1>League Tracker</h1>
        {/* <span>Leaderboard</span> */}
      </div>
      <div id="cardholder" class="flex flex-col gap-10 bg-slate-900">
        {games.map((game) => {
          return (
            <div key={game.id}>
              <MatchCard
                game={game}
                primary_rune={primary_runes.value[game.primary_rune]}
                items={vecItems(game)}
              />
            </div>
          );
        })}
      </div>
    </>
  );
});

export const head: DocumentHead = {
  title: "League Tracker",
  meta: [
    {
      name: "description",
      content: "A Simple Solo Queue Tracker for League of Legends",
    },
  ],
};
