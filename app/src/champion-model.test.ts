import { describe, expect, test } from "bun:test";

import { extractRecommendedBuild, filterChampions, roleLabel, type ChampionSummary } from "./champion-model";

const champions: ChampionSummary[] = [
  {
    id: "Twitch",
    key: "29",
    name: "Twitch",
    title: "Semeur de peste",
    tags: ["Marksman", "Assassin"],
    iconUrl: "twitch.png",
    version: "16.12.1",
  },
  {
    id: "Leona",
    key: "89",
    name: "Leona",
    title: "Aube radieuse",
    tags: ["Tank", "Support"],
    iconUrl: "leona.png",
    version: "16.12.1",
  },
];

describe("champion model", () => {
  test("filters champions by query and role", () => {
    expect(filterChampions(champions, "tw", "all").map((champion) => champion.id)).toEqual([
      "Twitch",
    ]);
    expect(filterChampions(champions, "", "Support").map((champion) => champion.id)).toEqual([
      "Leona",
    ]);
    expect(filterChampions(champions, "leo", "Marksman")).toEqual([]);
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
});
