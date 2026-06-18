import { useMemo } from "react";

import { useAppActions } from "../../application/useAppActions";
import {
  filterChampions,
  parsePercentFilter,
  roleLabel,
  type ChampionSummary,
  type ChampionTag,
} from "../../domain/champion";
import { useAppStore } from "../../store/appStore";

const championRoles: Array<ChampionTag | "all"> = [
  "all",
  "Assassin",
  "Fighter",
  "Mage",
  "Marksman",
  "Support",
  "Tank",
];

export function ChampionCatalogView(): React.JSX.Element {
  const { dispatch, state } = useAppStore();
  const { openChampionDetail } = useAppActions();
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
