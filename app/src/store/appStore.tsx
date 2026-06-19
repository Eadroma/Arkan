import { createContext, useContext, useMemo, useReducer, type Dispatch, type ReactNode } from "react";

import type { ChampionDetail, ChampionSummary, ChampionTag } from "../domain/champion";
import type { ChampionMastery, LeagueClientCard, PlayerProfile } from "../domain/league";

export type ViewName = "champion-detail" | "champions" | "profile";

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
  playerProfile: PlayerProfile;
  search: SearchState;
  selectedChampion?: ChampionDetail;
  selectedChampionRole: string;
  sidebarCollapsed: boolean;
  view: ViewName;
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
