import type { GameAssets, RuneTree, SummonerSpell } from "../domain/assets";
import type { ChampionDetail, ChampionSummary } from "../domain/champion";

type DataDragonChampion = {
  id: string;
  image: {
    full: string;
  };
  key: string;
  name: string;
  passive?: ChampionDetail["passive"];
  recommended?: ChampionDetail["recommended"];
  spells?: ChampionDetail["spells"];
  tags: ChampionSummary["tags"];
  title: string;
};

const dataDragonRoot = "https://ddragon.leagueoflegends.com";

let versionCache: string | null = null;
let championIndexCache: Map<string, ChampionDetail> | null = null;
let gameAssetsCache: GameAssets | null = null;

export async function latestDataDragonVersion(): Promise<string> {
  if (versionCache) {
    return versionCache;
  }

  const response = await fetch(`${dataDragonRoot}/api/versions.json`);
  const versions = (await response.json()) as string[];
  const [latestVersion] = versions;

  if (!latestVersion) {
    throw new Error("No Data Dragon version available");
  }

  versionCache = latestVersion;
  return latestVersion;
}

export async function loadChampionIndex(): Promise<Map<string, ChampionDetail>> {
  if (championIndexCache) {
    return championIndexCache;
  }

  const version = await latestDataDragonVersion();
  const response = await fetch(`${dataDragonRoot}/cdn/${version}/data/fr_FR/champion.json`);
  const championData = (await response.json()) as { data: Record<string, DataDragonChampion> };

  championIndexCache = new Map(
    Object.values(championData.data).map((champion) => [
      champion.id,
      {
        iconUrl: championIconUrl(version, champion.image.full),
        id: champion.id,
        key: champion.key,
        name: champion.name,
        recommended: [],
        spells: [],
        tags: champion.tags,
        title: champion.title,
        version,
      },
    ]),
  );

  return championIndexCache;
}

export async function loadChampionDetail(championId: string): Promise<ChampionDetail> {
  const index = await loadChampionIndex();
  const champion = index.get(championId);

  if (!champion) {
    throw new Error(`Champion inconnu: ${championId}`);
  }

  if (champion.spells.length > 0 || champion.recommended.length > 0) {
    return champion;
  }

  const response = await fetch(
    `${dataDragonRoot}/cdn/${champion.version}/data/fr_FR/champion/${champion.id}.json`,
  );
  const details = (await response.json()) as { data: Record<string, DataDragonChampion> };
  const detailedChampion = details.data[champion.id];
  const hydratedChampion: ChampionDetail = {
    ...champion,
    passive: detailedChampion.passive,
    recommended: detailedChampion.recommended ?? [],
    spells: detailedChampion.spells ?? [],
  };

  index.set(champion.id, hydratedChampion);
  return hydratedChampion;
}

export async function loadGameAssets(version: string): Promise<GameAssets> {
  if (gameAssetsCache) {
    return gameAssetsCache;
  }

  const [runeResponse, spellResponse] = await Promise.all([
    fetch(`${dataDragonRoot}/cdn/${version}/data/fr_FR/runesReforged.json`),
    fetch(`${dataDragonRoot}/cdn/${version}/data/fr_FR/summoner.json`),
  ]);
  const runeTrees = (await runeResponse.json()) as RuneTree[];
  const spellData = (await spellResponse.json()) as { data: Record<string, SummonerSpell> };

  gameAssetsCache = {
    runeTrees,
    summonerSpells: spellData.data,
  };

  return gameAssetsCache;
}

export function championIconUrl(version: string, image: string): string {
  return `${dataDragonRoot}/cdn/${version}/img/champion/${image}`;
}

export function profileIconUrl(version: string, profileIconId: number): string {
  return `${dataDragonRoot}/cdn/${version}/img/profileicon/${profileIconId}.png`;
}

export function spellIconUrl(version: string, image: string): string {
  return `${dataDragonRoot}/cdn/${version}/img/spell/${image}`;
}

export function itemIconUrl(version: string, itemId: number): string {
  return `${dataDragonRoot}/cdn/${version}/img/item/${itemId}.png`;
}

export function passiveIconUrl(version: string, image: string): string {
  return `${dataDragonRoot}/cdn/${version}/img/passive/${image}`;
}

export function runeIconUrl(icon: string): string {
  return `${dataDragonRoot}/cdn/img/${icon}`;
}
