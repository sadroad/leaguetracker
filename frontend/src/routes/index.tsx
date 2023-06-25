import { component$ } from "@builder.io/qwik";
import { routeLoader$, DocumentHead } from "@builder.io/qwik-city";
import { Game } from "../../../bindings/Game.ts";
import { MatchCard } from "../components/matchcard/matchcard";

interface Games {
  games: Game[];
}

export const getGameData = routeLoader$(async (requestEvent) => {
  const endpoint = requestEvent.env.get("API_SERVER")+"/api/games";
  const res = await fetch(endpoint);
  const data = await res.json() as Games;
  const games = data.games.sort((a, b) => {
    if (a.id === b.id) {
      return 0;
    }
    return a.id < b.id ? 1 : -1;
  });
  return games;
});

export const getPrimaryRunes = routeLoader$(async () => {
  const res = await fetch(
    "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/perks.json"
  );
  const rune_data = await res.json();
  const rune_map = rune_data.reduce(
    (
      acc: { [x: string]: any },
      rune: { iconPath: string; id: string | number }
    ) => {
      const rune_path = rune.iconPath
        .slice(rune.iconPath.indexOf("Styles") + 7)
        .toLowerCase();
      acc[rune.id] = rune_path;
      return acc;
    },
    {}
  );
  return rune_map;
});

export const getItems = routeLoader$(async () => {
  const res = await fetch(
    "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/items.json"
  );
  const item_data = await res.json();
  const item_map = item_data.reduce(
    (
      acc: { [x: string]: any },
      item: { id: string | number; iconPath: string }
    ) => {
      acc[item.id] = item.iconPath
        .slice(item.iconPath.indexOf("Icons2D") + 8)
        .toLowerCase();
      return acc;
    },
    {}
  );
  return item_map;
});

export default component$(() => {
  const games = getGameData();
  const primary_runes = getPrimaryRunes();
  const get_items = getItems();
  const vecItems = (game: Game) => {
    const items = [];
    for (let i = 0; i < 7; i++) {
      const item = game[`item_${i}` as keyof Game];
      items.push(get_items.value[item as string]);
    }
    return items;
  };

  return (
    <div class="bg-slate-800">
      <div id="navbar" class="text-5xl font-bold text-orange-200">
        <h1>League Tracker</h1>
        {/* <span>Leaderboard</span> */}
      </div>
      <div id="cardholder" class="flex flex-col gap-10 pt-10">
        {games.value.map((game) => {
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
    </div>
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
