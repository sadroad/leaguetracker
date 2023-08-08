import { component$ } from "@builder.io/qwik";
import type { DocumentHead } from "@builder.io/qwik-city";
import { routeLoader$ } from "@builder.io/qwik-city";
import type { Game } from "../../bindings/Game.ts";
import { MatchCard } from "../components/matchcard/matchcard";
import { itemsBlob } from "../utilities/items.ts";
import { perksBlob } from "~/utilities/perks.ts";

interface Games {
  games: Game[];
}

export const useGameData = routeLoader$(async (requestEvent) => {
  const endpoint = requestEvent.env.get("API_SERVER") + "/api/get_games";
  const res = await fetch(endpoint);
  const data = (await res.json()) as Games;
  const games = data.games.sort((a, b) => {
    if (a.id === b.id) {
      return 0;
    }
    return a.id < b.id ? 1 : -1;
  });
  return games;
});

export const usePrimaryRunes = routeLoader$(async () => {
  const rune_data = perksBlob;
  const rune_map = rune_data.reduce(
    (
      acc: Map<string | number, string>,
      rune: { iconPath: string; id: string | number }
    ) => {
      const rune_path = rune.iconPath
        .slice(rune.iconPath.indexOf("Styles") + 7)
        .toLowerCase();
      acc.set(rune.id, rune_path);
      return acc;
    },
    new Map()
  );
  return rune_map;
});

export const useItems = routeLoader$(async () => {
  const item_data = itemsBlob;
  const item_map = item_data.reduce(
    (
      acc: Map<string | number, string>,
      item: { id: string | number; iconPath: string }
    ) => {
      const item_path = item.iconPath
        .slice(item.iconPath.indexOf("Icons2D") + 8)
        .toLowerCase();
      acc.set(item.id, item_path);
      return acc;
    },
    new Map()
  );
  return item_map;
});

export default component$(() => {
  const games = useGameData();
  const primary_runes = usePrimaryRunes();
  const get_items = useItems();
  const vecItems = (game: Game) => {
    const items = [];
    for (let i = 0; i < 7; i++) {
      const item = game[`item_${i}` as keyof Game];
      items.push(get_items.value.get(item as string | number));
    }
    return items;
  };

  return (
    <div class="bg-slate-800 h-auto w-screen absolute">
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
                primary_rune={primary_runes.value.get(game.primary_rune) ?? ""}
                items={vecItems(game) as string[]}
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
