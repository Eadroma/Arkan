import { describe, expect, test } from "bun:test";

import {
  abilityDescription,
  extractRecommendedBuild,
  filterChampions,
  parsePercentFilter,
  roleLabel,
  type ChampionSummary,
} from "./domain/champion";

const champions: ChampionSummary[] = [
  {
    id: "Twitch",
    key: "29",
    name: "Twitch",
    title: "Semeur de peste",
    tags: ["Marksman", "Assassin"],
    iconUrl: "twitch.png",
    version: "16.12.1",
    stats: {
      winRate: 51.8,
      pickRate: 7.4,
    },
  },
  {
    id: "Leona",
    key: "89",
    name: "Leona",
    title: "Aube radieuse",
    tags: ["Tank", "Support"],
    iconUrl: "leona.png",
    version: "16.12.1",
    stats: {
      winRate: 49.2,
      pickRate: 3.1,
    },
  },
  {
    id: "Aatrox",
    key: "266",
    name: "Aatrox",
    title: "Epée des Darkin",
    tags: ["Fighter", "Tank"],
    iconUrl: "aatrox.png",
    version: "16.12.1",
  },
];

describe("champion model", () => {
  test("filters champions by query and role", () => {
    expect(
      filterChampions(champions, { query: "tw", role: "all" }).map((champion) => champion.id),
    ).toEqual(["Twitch"]);
    expect(
      filterChampions(champions, { query: "", role: "Support" }).map((champion) => champion.id),
    ).toEqual(["Leona"]);
    expect(filterChampions(champions, { query: "leo", role: "Marksman" })).toEqual([]);
  });

  test("filters champions by synced winrate and pickrate thresholds", () => {
    expect(
      filterChampions(champions, { query: "", role: "all", minWinRate: 50 }).map(
        (champion) => champion.id,
      ),
    ).toEqual(["Twitch"]);
    expect(
      filterChampions(champions, { query: "", role: "all", minPickRate: 3 }).map(
        (champion) => champion.id,
      ),
    ).toEqual(["Twitch", "Leona"]);
    expect(
      filterChampions(champions, { query: "", role: "all", minWinRate: 1 }).map(
        (champion) => champion.id,
      ),
    ).not.toContain("Aatrox");
  });

  test("parses optional percent filters", () => {
    expect(parsePercentFilter("")).toBeUndefined();
    expect(parsePercentFilter("51,5")).toBe(51.5);
    expect(parsePercentFilter("-1")).toBeUndefined();
    expect(parsePercentFilter("101")).toBeUndefined();
    expect(parsePercentFilter("abc")).toBeUndefined();
  });

  test("labels Riot champion tags for the French UI", () => {
    expect(roleLabel("Fighter")).toBe("Combattant");
    expect(roleLabel("Marksman")).toBe("Tireur");
    expect(roleLabel("BOTTOM")).toBe("BOTTOM");
  });

  test("extracts the first recommended item block", () => {
    expect(
      extractRecommendedBuild([
        {
          blocks: [
            {
              items: [{ id: "1055" }, { id: "3006" }, { id: "6672" }, { id: "3085" }],
            },
          ],
        },
      ]),
    ).toEqual(["1055", "3006", "6672", "3085"]);
  });

  test("cleans Data Dragon ability markup", () => {
    expect(
      abilityDescription({
        name: "Incineration",
        description: "<mainText>Inflige <physicalDamage>{{ damage }}</physicalDamage><br />Stun.</mainText>",
      }),
    ).toBe("Inflige\nStun.");
  });
});
