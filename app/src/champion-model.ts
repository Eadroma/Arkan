export type ChampionTag = "Assassin" | "Fighter" | "Mage" | "Marksman" | "Support" | "Tank";

export type ChampionSummary = {
  id: string;
  key: string;
  name: string;
  title: string;
  tags: ChampionTag[];
  iconUrl: string;
  version: string;
};

export type RecommendedItem = {
  id: string;
};

export type RecommendedBlock = {
  items?: RecommendedItem[];
};

export type RecommendedBuild = {
  blocks?: RecommendedBlock[];
};

export function filterChampions(
  champions: ChampionSummary[],
  query: string,
  role: ChampionTag | "all",
): ChampionSummary[] {
  const normalizedQuery = query.trim().toLocaleLowerCase("fr");

  return champions.filter((champion) => {
    const matchesQuery = champion.name.toLocaleLowerCase("fr").includes(normalizedQuery);
    const matchesRole = role === "all" || champion.tags.includes(role);

    return matchesQuery && matchesRole;
  });
}

export function roleLabel(role: ChampionTag | "ALL" | "all" | string): string {
  const labels: Record<string, string> = {
    ALL: "Tous roles",
    all: "Tous roles",
    Assassin: "Assassin",
    Fighter: "Combattant",
    Mage: "Mage",
    Marksman: "Tireur",
    Support: "Support",
    Tank: "Tank",
  };

  return labels[role] ?? role;
}

export function extractRecommendedBuild(recommended: RecommendedBuild[]): string[] {
  const firstBlock = recommended
    .flatMap((recommendation) => recommendation.blocks ?? [])
    .find((block) => Array.isArray(block.items) && block.items.length > 0);

  return (firstBlock?.items ?? []).slice(0, 6).map((item) => item.id);
}
