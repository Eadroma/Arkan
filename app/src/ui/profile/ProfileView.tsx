import { useEffect, useMemo } from "react";

import { useAppActions } from "../../application/useAppActions";
import type { ChampionDetail } from "../../domain/champion";
import type { ChampionMastery, MatchHistoryEntry } from "../../domain/league";
import { useAppStore } from "../../store/appStore";
import { EmptyLines } from "../components/EmptyLines";
import { StatusPill } from "../components/StatusPill";
import { SurfaceCard } from "../components/SurfaceCard";

export function ProfileView(): React.JSX.Element {
  const { state } = useAppStore();
  const { loadChampionCatalog, loadMatchHistoryForDisplayedPlayer } = useAppActions();

  useEffect(() => {
    if (state.championPool.length > 0) {
      void loadChampionCatalog();
    }
  }, [loadChampionCatalog, state.championPool.length]);

  useEffect(() => {
    void loadMatchHistoryForDisplayedPlayer();
  }, [loadMatchHistoryForDisplayedPlayer]);

  return (
    <section className="dashboard">
      <section className="hero-panel">
        <div className="player-placeholder">
          <div className="avatar-placeholder">
            {state.playerProfile.iconUrl ? <img src={state.playerProfile.iconUrl} alt="" /> : null}
            <span hidden={state.playerProfile.iconUrl !== undefined}>A</span>
          </div>
          <div>
            <p className="panel-kicker">{state.playerProfile.kicker}</p>
            <h2>{state.playerProfile.displayName}</h2>
          </div>
        </div>
        <div className="hero-stats" aria-label="Player summary placeholders">
          <div>
            <span>Solo/Duo</span>
            <strong>--</strong>
          </div>
          <div>
            <span>Winrate</span>
            <strong>--</strong>
          </div>
          <div>
            <span>Games</span>
            <strong>--</strong>
          </div>
        </div>
      </section>
      <section className="cards-grid">
        <SurfaceCard
          title="League Client"
          aside={<StatusPill variant={state.leagueClient.variant}>{state.leagueClient.pill}</StatusPill>}
        >
          <div className="client-details">
            <div>
              <span>Status</span>
              <strong>{state.leagueClient.status}</strong>
            </div>
            <div>
              <span>Region</span>
              <strong>{state.leagueClient.region}</strong>
            </div>
            <div>
              <span>Level</span>
              <strong>{state.leagueClient.level}</strong>
            </div>
          </div>
        </SurfaceCard>
        <SurfaceCard title="Champion pool" aside={<span className="muted">Top 5</span>}>
          <ChampionPool />
        </SurfaceCard>
        <SurfaceCard title="Match history" aside={<span className="muted">Derniers matchs</span>} wide>
          <MatchHistoryList />
        </SurfaceCard>
      </section>
    </section>
  );
}

function MatchHistoryList(): React.JSX.Element {
  const { state } = useAppStore();
  const { openMatchDetail } = useAppActions();
  const championIconByKey = useChampionIconByKey(state.championCatalog);

  if (state.matchHistory.status === "loading") {
    return <EmptyLines />;
  }

  if (state.matchHistory.status === "error") {
    return <span className="match-history-state">Historique indisponible pour le moment.</span>;
  }

  if (state.matchHistory.entries.length === 0) {
    return <span className="match-history-state">Aucun match recent a afficher.</span>;
  }

  return (
    <div className="match-history-list">
      {state.matchHistory.entries.map((entry) => (
        <MatchHistoryRow
          entry={entry}
          iconUrl={championIconByKey.get(entry.championId.toString())}
          key={entry.matchId}
          onOpen={() => {
            void openMatchDetail(entry.matchId);
          }}
        />
      ))}
    </div>
  );
}

function MatchHistoryRow({
  entry,
  iconUrl,
  onOpen,
}: {
  entry: MatchHistoryEntry;
  iconUrl?: string;
  onOpen: () => void;
}): React.JSX.Element {
  return (
    <button className="match-history-row" data-result={entry.win ? "win" : "loss"} type="button" onClick={onOpen}>
      <div className="match-history-row__result">
        {iconUrl ? <img src={iconUrl} alt="" /> : <span className="match-history-row__fallback">?</span>}
        <div>
          <strong>{entry.win ? "Victoire" : "Defaite"}</strong>
          <span>{queueLabel(entry.queueId)} - {entry.role}</span>
        </div>
      </div>
      <div>
        <strong>{entry.championName}</strong>
        <span>{formatDuration(entry.durationSeconds)}</span>
      </div>
      <div>
        <strong>{entry.kills}/{entry.deaths}/{entry.assists}</strong>
        <span>KDA</span>
      </div>
      <div className="match-history-row__lp">
        <strong>{formatLpDelta(entry.lpDelta)}</strong>
        <span>LP</span>
      </div>
    </button>
  );
}

function ChampionPool(): React.JSX.Element {
  const { openChampionDetail } = useAppActions();
  const { state } = useAppStore();
  const championRows = useChampionRows(state.championPool, state.championCatalog);

  if (state.championPool.length === 0) {
    return <EmptyLines />;
  }

  return (
    <div className="champion-list">
      {state.championCatalogStatus === "loading" ? (
        <span className="champion-list-hint">Synchronisation des noms de champions...</span>
      ) : null}
      {championRows.map((row) => (
        <button
          className="champion-row"
          disabled={!row.championId}
          key={row.id}
          type="button"
          onClick={() => {
            if (row.championId) {
              void openChampionDetail(row.championId);
            }
          }}
        >
          {row.iconUrl ? <img src={row.iconUrl} alt="" /> : <span className="champion-row__fallback">?</span>}
          <div>
            <strong>{row.name}</strong>
            <span>{row.subtitle}</span>
          </div>
        </button>
      ))}
    </div>
  );
}

function useChampionRows(
  masteries: ChampionMastery[],
  catalog: ChampionDetail[],
): Array<{
  championId?: string;
  iconUrl?: string;
  id: string;
  name: string;
  subtitle: string;
}> {
  return useMemo(
    () =>
      masteries.map((mastery, index) => {
        const id = mastery.championId?.toString() ?? `unknown-${index}`;
        const champion = catalog.find((item) => item.key === id);

        return {
          championId: champion?.id,
          iconUrl: champion?.iconUrl,
          id,
          name: champion?.name ?? `Champion ${id}`,
          subtitle: `M${mastery.championLevel ?? "-"} - ${formatPoints(mastery.championPoints)} pts`,
        };
      }),
    [catalog, masteries],
  );
}

function useChampionIconByKey(catalog: ChampionDetail[]): Map<string, string> {
  return useMemo(
    () => new Map(catalog.map((champion) => [champion.key, champion.iconUrl])),
    [catalog],
  );
}

function formatPoints(value?: number): string {
  return value === undefined ? "--" : new Intl.NumberFormat("fr-FR").format(value);
}

function formatLpDelta(value: number | null): string {
  if (value === null) {
    return "--";
  }

  return value > 0 ? `+${value}` : value.toString();
}

function formatDuration(seconds: number): string {
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;

  return `${minutes}:${remainingSeconds.toString().padStart(2, "0")}`;
}

function queueLabel(queueId: number): string {
  const labels: Record<number, string> = {
    400: "Normal Draft",
    420: "Ranked Solo/Duo",
    430: "Normal Blind",
    440: "Ranked Flex",
    450: "ARAM",
  };

  return labels[queueId] ?? `Queue ${queueId}`;
}
