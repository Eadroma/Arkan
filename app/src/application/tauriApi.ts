import type { LeagueClientStatus, MatchHistoryEntry, RiotAccount } from "../domain/league";
import type { MatchDetail } from "../domain/match";

type TauriInvoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

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

export async function matchHistory(input: string, platform: string): Promise<MatchHistoryEntry[]> {
  const invoke = tauriInvoke();

  if (!invoke) {
    return [];
  }

  return invoke<MatchHistoryEntry[]>("match_history", { input, platform });
}

export async function matchDetail(matchId: string, platform: string): Promise<MatchDetail> {
  const invoke = tauriInvoke();

  if (!invoke) {
    throw new Error("Tauri runtime is unavailable");
  }

  return invoke<MatchDetail>("match_detail", { matchId, platform });
}
