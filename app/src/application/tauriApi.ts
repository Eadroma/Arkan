import type { LeagueClientStatus, MatchHistoryEntry, RiotAccount } from "../domain/league";
import type { ChampionRoleStats, ChampionRunePageStats, ChampionSpellPairStats } from "../domain/champion";
import type { MatchDetail } from "../domain/match";

type TauriInvoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

export type ChampionSampleSyncResult = {
  fetchedMatches: number;
  requestedMatches: number;
  source: string;
};

export type TopChampionSampleSyncResult = {
  fetchedMatches: number;
  requestedMatchesPerSeed: number;
  seedsSynced: number;
  source: string;
  tier: string;
};

declare global {
  interface Window {
    __TAURI__?: {
      core?: {
        invoke: TauriInvoke;
      };
    };
  }
}

function tauriInvoke(): TauriInvoke | undefined {
  return window.__TAURI__?.core?.invoke;
}

export function hasTauriRuntime(): boolean {
  return tauriInvoke() !== undefined;
}

export async function leagueClientStatus(): Promise<LeagueClientStatus> {
  const invoke = tauriInvoke();

  if (!invoke) {
    return {
      cached: false,
      connected: false,
      detected: false,
    };
  }

  return invoke<LeagueClientStatus>("league_client_status");
}

export async function resolveRiotAccount(input: string, platform: string): Promise<RiotAccount> {
  const invoke = tauriInvoke();

  if (!invoke) {
    throw new Error("Tauri runtime unavailable");
  }

  return invoke<RiotAccount>("resolve_riot_account", { input, platform });
}

export async function matchHistory(
  input: string,
  platform: string,
  start: number,
  count: number,
): Promise<MatchHistoryEntry[]> {
  const invoke = tauriInvoke();

  if (!invoke) {
    return [];
  }

  return invoke<MatchHistoryEntry[]>("match_history", { count, input, platform, start });
}

export async function syncChampionSample(
  input: string,
  platform: string,
  requestedMatches = 500,
): Promise<ChampionSampleSyncResult | undefined> {
  const invoke = tauriInvoke();

  if (!invoke) {
    return undefined;
  }

  return invoke<ChampionSampleSyncResult>("sync_champion_sample", { input, platform, requestedMatches });
}

export async function syncTopChampionSample(
  platform: string,
  tier = "challenger",
  seedCount = 1,
  requestedMatchesPerSeed = 1,
): Promise<TopChampionSampleSyncResult | undefined> {
  const invoke = tauriInvoke();

  if (!invoke) {
    return undefined;
  }

  return invoke<TopChampionSampleSyncResult>("sync_top_champion_sample", {
    platform,
    requestedMatchesPerSeed,
    seedCount,
    tier,
  });
}

export async function matchDetail(matchId: string, platform: string): Promise<MatchDetail> {
  const invoke = tauriInvoke();

  if (!invoke) {
    throw new Error("Tauri runtime is unavailable");
  }

  return invoke<MatchDetail>("match_detail", { matchId, platform });
}

export async function refreshChampionRoleStats(platform: string, tier?: string): Promise<number> {
  const invoke = tauriInvoke();

  if (!invoke) {
    return 0;
  }

  return invoke<number>("refresh_champion_role_stats", { platform, tier });
}

export async function championRoleStats(championId: number, platform: string): Promise<ChampionRoleStats[]> {
  const invoke = tauriInvoke();

  if (!invoke) {
    return [];
  }

  return invoke<ChampionRoleStats[]>("champion_role_stats", { championId, platform });
}

export async function championSpellPairs(championId: number): Promise<ChampionSpellPairStats[]> {
  const invoke = tauriInvoke();

  if (!invoke) {
    return [];
  }

  return invoke<ChampionSpellPairStats[]>("champion_spell_pairs", { championId });
}

export async function championRunePages(championId: number): Promise<ChampionRunePageStats[]> {
  const invoke = tauriInvoke();

  if (!invoke) {
    return [];
  }

  return invoke<ChampionRunePageStats[]>("champion_rune_pages", { championId });
}
