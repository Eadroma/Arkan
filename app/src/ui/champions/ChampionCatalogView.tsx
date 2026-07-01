import { useMemo, useState } from "react";

import { syncTopChampionSample, type TopChampionSampleSyncResult } from "../../application/tauriApi";
import { useAppActions } from "../../application/useAppActions";
import {
  filterChampions,
  parsePercentFilter,
  roleLabel,
  type ChampionSummary,
  type ChampionTag,
} from "../../domain/champion";
import { useAppStore } from "../../store/appStore";
import { Button } from "../components/Button";

const championRoles: Array<ChampionTag | "all"> = [
  "all",
  "Assassin",
  "Fighter",
  "Mage",
  "Marksman",
  "Support",
  "Tank",
];

type SyncStatus = "idle" | "loading" | "success" | "error";

export function ChampionCatalogView(): React.JSX.Element {
  const { dispatch, state } = useAppStore();
  const { openChampionDetail } = useAppActions();
  const [syncStatus, setSyncStatus] = useState<SyncStatus>("idle");
  const [syncTier, setSyncTier] = useState("challenger");
  const [syncSeedCount, setSyncSeedCount] = useState(1);
  const [syncMatchesPerSeed, setSyncMatchesPerSeed] = useState(1);
  const [syncError, setSyncError] = useState<string | undefined>();
  const [syncResult, setSyncResult] = useState<TopChampionSampleSyncResult | undefined>();
  const filters = state.championCatalogFilters;
  const minWinRate = parsePercentFilter(filters.minWinRate);
  const minPickRate = parsePercentFilter(filters.minPickRate);
  const champions = useMemo(
    () =>
      filterChampions(state.championCatalog, {
        minPickRate,
        minWinRate,
        query: filters.query,
        role: filters.role,
      }),
    [filters.query, filters.role, minPickRate, minWinRate, state.championCatalog],
  );
  const syncMessage = syncMessageFromState(syncStatus, syncResult, syncError);

  async function handleTopPlayerSync(): Promise<void> {
    setSyncStatus("loading");
    setSyncError(undefined);
    setSyncResult(undefined);

    try {
      const result = await syncTopChampionSample(
        state.search.region,
        syncTier,
        syncSeedCount,
        syncMatchesPerSeed,
      );
      setSyncResult(result);
      setSyncStatus(result ? "success" : "error");
      if (!result) {
        setSyncError("Tauri runtime indisponible.");
      }
    } catch (error) {
      setSyncError(error instanceof Error ? error.message : String(error));
      setSyncStatus("error");
    }
  }

  return (
    <section className="dashboard">
      <section className="section-head">
        <div>
          <p className="panel-kicker">Data Dragon</p>
          <h2>Champions</h2>
        </div>
        <div className="champion-filters">
          <div className="catalog-search">
            <span className="search-glyph" aria-hidden="true" />
            <input
              aria-label="Filtrer les champions"
              placeholder="Chercher un champion"
              value={filters.query}
              onChange={(event) =>
                dispatch({
                  filters: { query: event.currentTarget.value },
                  type: "championFiltersChanged",
                })
              }
            />
          </div>
          <select
            aria-label="Filtrer par role"
            value={filters.role}
            onChange={(event) =>
              dispatch({
                filters: { role: event.currentTarget.value as ChampionTag | "all" },
                type: "championFiltersChanged",
              })
            }
          >
            {championRoles.map((role) => (
              <option key={role} value={role}>{roleLabel(role)}</option>
            ))}
          </select>
          <input
            aria-label="Winrate minimum"
            inputMode="decimal"
            placeholder="Winrate %"
            title="Filtre les stats synchronisees MATCH-V5"
            value={filters.minWinRate}
            onChange={(event) =>
              dispatch({
                filters: { minWinRate: event.currentTarget.value },
                type: "championFiltersChanged",
              })
            }
          />
          <input
            aria-label="Pickrate minimum"
            inputMode="decimal"
            placeholder="Pickrate %"
            title="Filtre les stats synchronisees MATCH-V5"
            value={filters.minPickRate}
            onChange={(event) =>
              dispatch({
                filters: { minPickRate: event.currentTarget.value },
                type: "championFiltersChanged",
              })
            }
          />
        </div>
      </section>
      <section className="top-player-sync-panel" aria-label="Synchronisation top players">
        <div>
          <p className="panel-kicker">MATCH-V5 sample</p>
          <strong>Top player seeds</strong>
        </div>
        <div className="top-player-sync-controls">
          <select
            aria-label="Tier top player"
            value={syncTier}
            onChange={(event) => setSyncTier(event.currentTarget.value)}
          >
            <option value="challenger">Challenger</option>
            <option value="grandmaster">Grandmaster</option>
            <option value="master">Master</option>
          </select>
          <label>
            <span>Seeds</span>
            <input
              aria-label="Nombre de seeds"
              max={3}
              min={1}
              type="number"
              value={syncSeedCount}
              onChange={(event) => setSyncSeedCount(clampInteger(event.currentTarget.value, 1, 3))}
            />
          </label>
          <label>
            <span>Matches</span>
            <input
              aria-label="Matches par seed"
              max={25}
              min={1}
              step={1}
              type="number"
              value={syncMatchesPerSeed}
              onChange={(event) => setSyncMatchesPerSeed(clampInteger(event.currentTarget.value, 1, 25))}
            />
          </label>
          <Button disabled={syncStatus === "loading"} onClick={() => void handleTopPlayerSync()}>
            {syncStatus === "loading" ? "Sync..." : "Sync top"}
          </Button>
        </div>
        <span className="top-player-sync-status" data-status={syncStatus}>{syncMessage}</span>
      </section>
      <section className="champion-catalog">
        {champions.length > 0 ? (
          champions.map((champion) => (
            <ChampionTile
              champion={champion}
              key={champion.id}
              onOpen={() => void openChampionDetail(champion.id)}
            />
          ))
        ) : (
          <CatalogEmptyState hasStatFilter={minWinRate !== undefined || minPickRate !== undefined} />
        )}
      </section>
    </section>
  );
}

function clampInteger(value: string, min: number, max: number): number {
  const parsedValue = Number.parseInt(value, 10);

  if (Number.isNaN(parsedValue)) {
    return min;
  }

  return Math.min(Math.max(parsedValue, min), max);
}

function syncMessageFromState(
  status: SyncStatus,
  result: TopChampionSampleSyncResult | undefined,
  error?: string,
): string {
  if (status === "loading") {
    return "Synchronisation en cours. Les stats se mettront a jour apres ingestion.";
  }

  if (status === "error") {
    return syncErrorMessage(error);
  }

  if (status === "success" && result) {
    return `${result.seedsSynced} seeds, ${result.fetchedMatches} matches ajoutes (${result.tier}).`;
  }

  return "Utilise les top ladders Riot officiels avec des limites prudentes.";
}

function syncErrorMessage(error: string | undefined): string {
  if (!error) {
    return "Sync impossible pour le moment.";
  }

  if (error.includes("HTTP 429")) {
    return "Rate limit Riot atteinte. Attends 1-2 minutes puis retente avec 1 seed / 1 match.";
  }

  return `Sync impossible: ${error}`;
}

function ChampionTile({
  champion,
  onOpen,
}: {
  champion: ChampionSummary;
  onOpen: () => void;
}): React.JSX.Element {
  return (
    <article className="champion-tile-wrap">
      <button className="champion-tile" type="button" onClick={onOpen}>
        <img alt="" src={champion.iconUrl} />
        <strong>{champion.name}</strong>
        <span>{champion.title}</span>
        <span className="champion-tile-stats">
          {champion.stats?.winRate !== undefined || champion.stats?.pickRate !== undefined
            ? `${formatOptionalPercent(champion.stats.winRate)} WR | ${formatOptionalPercent(champion.stats.pickRate)} PR`
            : "Stats a synchroniser"}
        </span>
      </button>
    </article>
  );
}

function CatalogEmptyState({ hasStatFilter }: { hasStatFilter: boolean }): React.JSX.Element {
  return (
    <div className="catalog-empty-state">
      {hasStatFilter
        ? "Aucun champion avec ces seuils. Les filtres winrate/pickrate utiliseront les agregats MATCH-V5 synchronises."
        : "Aucun champion ne correspond a ces filtres."}
    </div>
  );
}

function formatOptionalPercent(value?: number): string {
  return value === undefined ? "--" : `${value.toFixed(1)}%`;
}
