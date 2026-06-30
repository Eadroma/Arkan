export type ChampionTag = "Assassin" | "Fighter" | "Mage" | "Marksman" | "Support" | "Tank";

export type ChampionStats = {
  pickRate?: number;
  winRate?: number;
};

export type ChampionRoleStats = {
  championId: number;
  championKey: string;
  championName: string;
  patch: string;
  pickRate: number;
  platformId: string;
  queueId: number;
  role: string;
  sampleSize: number;
  source: string;
  tier?: string;
  winRate: number;
  wins: number;
};

export type ChampionSummary = {
  iconUrl: string;
  id: string;
  key: string;
  name: string;
  stats?: ChampionStats;
  tags: ChampionTag[];
  title: string;
  version: string;
};

export type ChampionSpell = {
  cooldownBurn?: string;
  costBurn?: string;
  description?: string;
  image?: { full: string };
  name: string;
  rangeBurn?: string;
  tooltip?: string;
};

export type ChampionDetail = ChampionSummary & {
  passive?: { description?: string; image?: { full: string }; name: string };
  recommended: RecommendedBuild[];
  spells: ChampionSpell[];
};

export type ChampionFilters = {
  minPickRate?: number;
  minWinRate?: number;
  query: string;
  role: ChampionTag | "all";
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
  filters: ChampionFilters,
): ChampionSummary[] {
  const normalizedQuery = filters.query.trim().toLocaleLowerCase("fr");

  return champions.filter((champion) => {
    const matchesQuery = champion.name.toLocaleLowerCase("fr").includes(normalizedQuery);
    const matchesRole = filters.role === "all" || champion.tags.includes(filters.role);
    const matchesWinRate =
      filters.minWinRate === undefined ||
      (champion.stats?.winRate !== undefined && champion.stats.winRate >= filters.minWinRate);
    const matchesPickRate =
      filters.minPickRate === undefined ||
      (champion.stats?.pickRate !== undefined && champion.stats.pickRate >= filters.minPickRate);

    return matchesQuery && matchesRole && matchesWinRate && matchesPickRate;
  });
}

export function parsePercentFilter(value: string): number | undefined {
  const normalizedValue = value.trim().replace(",", ".");

  if (normalizedValue.length === 0) {
    return undefined;
  }

  const parsedValue = Number(normalizedValue);

  if (!Number.isFinite(parsedValue) || parsedValue < 0 || parsedValue > 100) {
    return undefined;
  }

  return parsedValue;
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

export function abilityDescription(spell: ChampionSpell): string {
  return stripDataDragonMarkup(spell.description || spell.tooltip || "");
}

export function stripDataDragonMarkup(value: string): string {
  return value
    .replace(/<br\s*\/?>/gi, "\n")
    .replace(/<[^>]+>/g, "")
    .replace(/{{\s*[^}]+\s*}}/g, "")
    .replace(/\s+\n/g, "\n")
    .replace(/\n{3,}/g, "\n\n")
    .replace(/[ \t]{2,}/g, " ")
    .trim();
}
