import { useCallback } from "react";

import { latestDataDragonVersion, loadChampionDetail, loadChampionIndex, profileIconUrl } from "./dataDragonApi";
import { hasTauriRuntime, leagueClientStatus, matchDetail, matchHistory, resolveRiotAccount } from "./tauriApi";
import type { ChampionDetail } from "../domain/champion";
import type { ChampionMastery, LeagueClientStatus, RiotAccount } from "../domain/league";
import { useAppStore, type ViewName } from "../store/appStore";

export function useAppActions(): {
  detectLeagueClient: () => Promise<void>;
  loadChampionCatalog: () => Promise<void>;
  loadMatchHistoryForDisplayedPlayer: () => Promise<void>;
  openChampionDetail: (championId: string) => Promise<void>;
  openMatchDetail: (matchId: string) => Promise<void>;
  resetToConnectedPlayer: () => void;
  searchRiotAccount: () => Promise<void>;
  setView: (view: ViewName) => Promise<void>;
} {
  const { dispatch, state } = useAppStore();

  const loadChampionCatalog = useCallback(async () => {
    if (state.championCatalog.length > 0 || state.championCatalogStatus === "loading") {
      return;
    }

    dispatch({ status: "loading", type: "championCatalogStatusChanged" });

    try {
      const index = await loadChampionIndex();
      const champions = [...index.values()].sort((first, second) =>
        first.name.localeCompare(second.name, "fr"),
      );
      dispatch({ champions, type: "championCatalogLoaded" });
    } catch {
      dispatch({ status: "error", type: "championCatalogStatusChanged" });
    }
  }, [dispatch, state.championCatalog.length, state.championCatalogStatus]);

  const openChampionDetail = useCallback(
    async (championId: string) => {
      await loadChampionCatalog();
      const champion = await loadChampionDetail(championId);
      dispatch({ champion, type: "championDetailSelected" });
    },
    [dispatch, loadChampionCatalog],
  );

  const setView = useCallback(
    async (view: ViewName) => {
      if (view === "champions" || view === "champion-detail") {
        await loadChampionCatalog();
      }

      dispatch({ type: "viewChanged", view });
    },
    [dispatch, loadChampionCatalog],
  );

  const detectLeagueClient = useCallback(async () => {
    dispatch({
      card: {
        level: "--",
        pill: "Scanning",
        region: state.search.region,
        status: "Checking",
        variant: "loading",
      },
      type: "leagueClientChanged",
    });

    if (!hasTauriRuntime()) {
      dispatch({
        card: {
          level: "--",
          pill: "Preview",
          region: state.search.region,
          status: "Preview",
          variant: "warning",
        },
        type: "leagueClientChanged",
      });
      return;
    }

    const status = await leagueClientStatus();
    await applyLeagueClientStatus(status, dispatch, state.search.region);
  }, [dispatch, state.search.region]);

  const loadMatchHistoryForDisplayedPlayer = useCallback(async () => {
    const riotId = riotIdFromDisplayName(state.playerProfile.displayName);

    if (!riotId) {
      dispatch({ entries: [], type: "matchHistoryLoaded" });
      return;
    }

    dispatch({ status: "loading", type: "matchHistoryStatusChanged" });

    try {
      const entries = await matchHistory(riotId, state.playerProfile.region || state.search.region);
      dispatch({ entries, type: "matchHistoryLoaded" });
    } catch {
      dispatch({ status: "error", type: "matchHistoryStatusChanged" });
    }
  }, [dispatch, state.playerProfile.displayName, state.playerProfile.region, state.search.region]);

  const openMatchDetail = useCallback(
    async (matchId: string) => {
      dispatch({ matchId, type: "matchDetailLoadingStarted" });

      try {
        await loadChampionCatalog();
        const detail = await matchDetail(matchId, state.playerProfile.region || state.search.region);
        dispatch({ detail, type: "matchDetailLoaded" });
      } catch {
        dispatch({ type: "matchDetailLoadingFailed" });
      }
    },
    [dispatch, loadChampionCatalog, state.playerProfile.region, state.search.region],
  );

  const searchRiotAccount = useCallback(async () => {
    const input = state.search.input.trim();

    if (!input) {
      return;
    }

    dispatch({ search: { isPending: true }, type: "searchChanged" });

    try {
      const [account, version] = await Promise.all([
        resolveRiotAccount(input, state.search.region),
        latestDataDragonVersion(),
      ]);
      dispatch({
        profile: playerProfileFromRiotAccount(account, state.search.region, version),
        type: "playerProfileChanged",
      });
      dispatch({ pool: account.championMasteries ?? [], type: "championPoolChanged" });
      dispatch({
        card: {
          level: account.summonerLevel?.toString() ?? "--",
          pill: "Resolved",
          region: state.search.region,
          status: "Resolved",
          variant: "online",
        },
        type: "leagueClientChanged",
      });
    } finally {
      dispatch({ search: { isPending: false }, type: "searchChanged" });
    }
  }, [dispatch, state.search.input, state.search.region]);

  const resetToConnectedPlayer = useCallback(() => {
    dispatch({ search: { input: "" }, type: "searchChanged" });
    dispatch({ profile: state.connectedPlayerProfile, type: "playerProfileChanged" });
    dispatch({ pool: state.connectedChampionPool, type: "championPoolChanged" });
    dispatch({ type: "viewChanged", view: "profile" });
  }, [dispatch, state.connectedChampionPool, state.connectedPlayerProfile]);

  return {
    detectLeagueClient,
    loadChampionCatalog,
    loadMatchHistoryForDisplayedPlayer,
    openChampionDetail,
    openMatchDetail,
    resetToConnectedPlayer,
    searchRiotAccount,
    setView,
  };
}

function riotIdFromDisplayName(displayName: string): string | undefined {
  const normalized = displayName.trim();

  return normalized.includes("#") ? normalized : undefined;
}

async function applyLeagueClientStatus(
  status: LeagueClientStatus,
  dispatch: ReturnType<typeof useAppStore>["dispatch"],
  region: string,
): Promise<void> {
  if (!status.detected) {
    dispatch({
      card: {
        level: "--",
        pill: "Offline",
        region,
        status: "Not detected",
        variant: "offline",
      },
      type: "leagueClientChanged",
    });
    return;
  }

  if (!status.connected || !status.summoner) {
    dispatch({
      card: {
        level: "--",
        pill: "Detected",
        region,
        status: "Detected",
        variant: "warning",
      },
      type: "leagueClientChanged",
    });
    return;
  }

  const summoner = status.summoner;
  const displayName = summoner.gameName && summoner.tagLine
    ? `${summoner.gameName}#${summoner.tagLine}`
    : summoner.displayName;
  const profile = {
    championMasteries: summoner.championMasteries ?? [],
    displayName,
    kicker: "Joueur connecte detecte",
    level: summoner.summonerLevel?.toString() ?? "--",
    region,
    status: "Detected",
  };

  dispatch({
    pool: summoner.championMasteries ?? [],
    profile,
    type: "connectedPlayerChanged",
  });
  dispatch({
    card: {
      level: summoner.summonerLevel?.toString() ?? "--",
      pill: status.cached ? "Cached" : "Detected",
      region,
      status: "Detected",
      variant: "online",
    },
    type: "leagueClientChanged",
  });

  if (summoner.profileIconId === undefined) {
    return;
  }

  try {
    const version = await latestDataDragonVersion();
    dispatch({
      pool: summoner.championMasteries ?? [],
      profile: {
        ...profile,
        iconUrl: profileIconUrl(version, summoner.profileIconId),
      },
      type: "connectedPlayerChanged",
    });
  } catch {
    // Keep the detected player and champion pool even if Data Dragon is temporarily unavailable.
  }
}

function playerProfileFromRiotAccount(
  account: RiotAccount,
  region: string,
  version: string,
): {
  championMasteries: ChampionMastery[];
  displayName: string;
  iconUrl?: string;
  kicker: string;
  level: string;
  region: string;
  status: string;
} {
  return {
    championMasteries: account.championMasteries ?? [],
    displayName: `${account.gameName}#${account.tagLine}`,
    iconUrl:
      account.profileIconId === undefined ? undefined : profileIconUrl(version, account.profileIconId),
    kicker: "Compte Riot resolu",
    level: account.summonerLevel?.toString() ?? "--",
    region,
    status: "Resolved",
  };
}
