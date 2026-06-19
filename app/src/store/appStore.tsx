import { createContext, useContext, useMemo, useReducer, type Dispatch, type ReactNode } from "react";

import type { ChampionDetail, ChampionSummary, ChampionTag } from "../domain/champion";
import type { ChampionMastery, LeagueClientCard, MatchHistoryEntry, PlayerProfile } from "../domain/league";
import type { MatchDetail } from "../domain/match";

export type ViewName = "champion-detail" | "champions" | "match-detail" | "profile";

export type ChampionCatalogFilters = {
  minPickRate: string;
  minWinRate: string;
  query: string;
  role: ChampionTag | "all";
};

export type SearchState = {
  input: string;
  isPending: boolean;
  region: string;
};

export type AbilityPanelState = {
  abilityKey?: string;
};

export type AppState = {
  abilityPanel: AbilityPanelState;
  championCatalog: ChampionDetail[];
  championCatalogFilters: ChampionCatalogFilters;
  championCatalogStatus: "idle" | "loading" | "ready" | "error";
  connectedPlayerProfile: PlayerProfile;
  championPool: ChampionMastery[];
  connectedChampionPool: ChampionMastery[];
  leagueClient: LeagueClientCard;
  matchHistory: MatchHistoryState;
  matchDetail: MatchDetailState;
  playerProfile: PlayerProfile;
  search: SearchState;
  selectedChampion?: ChampionDetail;
  selectedChampionRole: string;
  sidebarCollapsed: boolean;
  view: ViewName;
};

export type MatchHistoryState = {
  entries: MatchHistoryEntry[];
  status: "idle" | "loading" | "ready" | "error";
};

export type MatchDetailState = {
  detail?: MatchDetail;
  matchId?: string;
  status: "idle" | "loading" | "ready" | "error";
};

export type AppAction =
  | { type: "abilityPanelClosed" }
  | { abilityKey: string; type: "abilityPanelToggled" }
  | { champion: ChampionDetail; type: "championDetailSelected" }
  | { champions: ChampionDetail[]; type: "championCatalogLoaded" }
  | { status: AppState["championCatalogStatus"]; type: "championCatalogStatusChanged" }
  | { filters: Partial<ChampionCatalogFilters>; type: "championFiltersChanged" }
  | { pool: ChampionMastery[]; profile: PlayerProfile; type: "connectedPlayerChanged" }
  | { pool: ChampionMastery[]; type: "championPoolChanged" }
  | { profile: PlayerProfile; type: "playerProfileChanged" }
  | { card: LeagueClientCard; type: "leagueClientChanged" }
  | { entries: MatchHistoryEntry[]; type: "matchHistoryLoaded" }
  | { status: MatchHistoryState["status"]; type: "matchHistoryStatusChanged" }
  | { detail: MatchDetail; type: "matchDetailLoaded" }
  | { matchId: string; type: "matchDetailLoadingStarted" }
  | { type: "matchDetailLoadingFailed" }
  | { role: string; type: "selectedChampionRoleChanged" }
  | { search: Partial<SearchState>; type: "searchChanged" }
  | { collapsed: boolean; type: "sidebarChanged" }
  | { view: ViewName; type: "viewChanged" };

const defaultLeagueClient: LeagueClientCard = {
  level: "--",
  pill: "Scanning",
  region: "EUW1",
  status: "Checking",
  variant: "loading",
};

const defaultPlayerProfile: PlayerProfile = {
  championMasteries: [],
  displayName: "Recherche automatique...",
  kicker: "Detection client League",
  level: "--",
  region: "EUW1",
  status: "Checking",
};

const initialState: AppState = {
  abilityPanel: {},
  championCatalog: [],
  championCatalogFilters: {
    minPickRate: "",
    minWinRate: "",
    query: "",
    role: "all",
  },
  championCatalogStatus: "idle",
  connectedPlayerProfile: defaultPlayerProfile,
  championPool: [],
  connectedChampionPool: [],
  leagueClient: defaultLeagueClient,
  matchHistory: {
    entries: [],
    status: "idle",
  },
  matchDetail: {
    status: "idle",
  },
  playerProfile: defaultPlayerProfile,
  search: {
    input: "",
    isPending: false,
    region: "EUW1",
  },
  selectedChampionRole: "ALL",
  sidebarCollapsed: localStorage.getItem("arkan.sidebar") === "collapsed",
  view: "profile",
};

type AppStoreValue = {
  dispatch: Dispatch<AppAction>;
  state: AppState;
};

const AppStoreContext = createContext<AppStoreValue | null>(null);

export function AppStoreProvider({ children }: { children: ReactNode }): ReactNode {
  const [state, dispatch] = useReducer(appReducer, initialState);
  const value = useMemo(() => ({ dispatch, state }), [state]);

  return <AppStoreContext.Provider value={value}>{children}</AppStoreContext.Provider>;
}

export function useAppStore(): AppStoreValue {
  const store = useContext(AppStoreContext);

  if (!store) {
    throw new Error("useAppStore must be used inside AppStoreProvider");
  }

  return store;
}

function appReducer(state: AppState, action: AppAction): AppState {
  switch (action.type) {
    case "abilityPanelClosed":
      return {
        ...state,
        abilityPanel: {},
      };
    case "abilityPanelToggled":
      return {
        ...state,
        abilityPanel:
          state.abilityPanel.abilityKey === action.abilityKey ? {} : { abilityKey: action.abilityKey },
      };
    case "championCatalogLoaded":
      return {
        ...state,
        championCatalog: action.champions,
        championCatalogStatus: "ready",
      };
    case "championCatalogStatusChanged":
      return {
        ...state,
        championCatalogStatus: action.status,
      };
    case "championDetailSelected":
      return {
        ...state,
        abilityPanel: {},
        selectedChampion: action.champion,
        selectedChampionRole: action.champion.tags[0] ?? "ALL",
        view: "champion-detail",
      };
    case "championFiltersChanged":
      return {
        ...state,
        championCatalogFilters: {
          ...state.championCatalogFilters,
          ...action.filters,
        },
      };
    case "connectedPlayerChanged":
      return {
        ...state,
        championPool: action.pool,
        connectedChampionPool: action.pool,
        connectedPlayerProfile: action.profile,
        playerProfile: action.profile,
      };
    case "championPoolChanged":
      return {
        ...state,
        championPool: action.pool,
      };
    case "leagueClientChanged":
      return {
        ...state,
        leagueClient: action.card,
      };
    case "matchHistoryLoaded":
      return {
        ...state,
        matchHistory: {
          entries: action.entries,
          status: "ready",
        },
      };
    case "matchHistoryStatusChanged":
      return {
        ...state,
        matchHistory: {
          ...state.matchHistory,
          entries: action.status === "loading" ? [] : state.matchHistory.entries,
          status: action.status,
        },
      };
    case "matchDetailLoadingStarted":
      return {
        ...state,
        matchDetail: {
          matchId: action.matchId,
          status: "loading",
        },
        view: "match-detail",
      };
    case "matchDetailLoaded":
      return {
        ...state,
        matchDetail: {
          detail: action.detail,
          matchId: action.detail.matchId,
          status: "ready",
        },
        view: "match-detail",
      };
    case "matchDetailLoadingFailed":
      return {
        ...state,
        matchDetail: {
          ...state.matchDetail,
          status: "error",
        },
        view: "match-detail",
      };
    case "playerProfileChanged":
      return {
        ...state,
        playerProfile: action.profile,
      };
    case "searchChanged":
      return {
        ...state,
        search: {
          ...state.search,
          ...action.search,
        },
      };
    case "selectedChampionRoleChanged":
      return {
        ...state,
        abilityPanel: {},
        selectedChampionRole: action.role,
      };
    case "sidebarChanged":
      localStorage.setItem("arkan.sidebar", action.collapsed ? "collapsed" : "expanded");
      return {
        ...state,
        sidebarCollapsed: action.collapsed,
      };
    case "viewChanged":
      return {
        ...state,
        view: action.view,
      };
  }
}
