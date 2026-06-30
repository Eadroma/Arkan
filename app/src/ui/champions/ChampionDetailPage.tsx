import { useEffect, useMemo, useState, type CSSProperties } from "react";

import {
  loadGameAssets,
  passiveIconUrl,
  runeIconUrl,
  spellIconUrl,
} from "../../application/dataDragonApi";
import type { GameAssets, RuneTree, SummonerSpell } from "../../domain/assets";
import {
  abilityDescription,
  extractRecommendedBuild,
  roleLabel,
  type ChampionDetail,
  type ChampionRunePageStats,
  type ChampionRoleStats,
  type ChampionSpellPairStats,
} from "../../domain/champion";
import {
  championRunePages as loadChampionRunePages,
  championRoleStats as loadChampionRoleStats,
  championSpellPairs as loadChampionSpellPairs,
} from "../../application/tauriApi";
import { useAppActions } from "../../application/useAppActions";
import { useAppStore } from "../../store/appStore";
import { Button } from "../components/Button";

const spellKeys = ["Q", "W", "E", "R"] as const;

type AbilityInfo = {
  cooldown?: string;
  cost?: string;
  description: string;
  iconUrl: string;
  key: string;
  name: string;
  range?: string;
};

export function ChampionDetailPage({ champion }: { champion: ChampionDetail }): React.JSX.Element {
  const { state } = useAppStore();
  const { setView } = useAppActions();
  const [assets, setAssets] = useState<GameAssets | null>(null);
  const [localStats, setLocalStats] = useState<ChampionRoleStats[]>([]);
  const [runePages, setRunePages] = useState<ChampionRunePageStats[]>([]);
  const [spellPairs, setSpellPairs] = useState<ChampionSpellPairStats[]>([]);
  const role = state.selectedChampionRole;
  const abilities = useChampionAbilities(champion);
  const selectedAbility = abilities.find((ability) => ability.key === state.abilityPanel.abilityKey);
  const selectedStats = selectChampionRoleStats(localStats, role);
  const selectedRunePage = runePages[0];
  const selectedSpellPair = spellPairs[0];

  useEffect(() => {
    void loadGameAssets(champion.version).then(setAssets);
  }, [champion.version]);

  useEffect(() => {
    setLocalStats([]);
    void loadChampionRoleStats(Number(champion.key), state.playerProfile.region || state.search.region)
      .then(setLocalStats)
      .catch(() => setLocalStats([]));
  }, [champion.key, state.playerProfile.region, state.search.region]);

  useEffect(() => {
    setSpellPairs([]);
    void loadChampionSpellPairs(Number(champion.key))
      .then(setSpellPairs)
      .catch(() => setSpellPairs([]));
  }, [champion.key]);

  useEffect(() => {
    setRunePages([]);
    void loadChampionRunePages(Number(champion.key))
      .then(setRunePages)
      .catch(() => setRunePages([]));
  }, [champion.key]);

  return (
    <section className="dashboard champion-build-page">
      <ChampionHero
        abilities={abilities}
        champion={champion}
        role={role}
        onBack={() => void setView("champions")}
      />
      {selectedAbility ? <AbilityPanel ability={selectedAbility} /> : null}
      <nav className="build-tabs" aria-label="Champion build tabs">
        <button className="active" type="button">Build</button>
        <button type="button">Counters</button>
        <button type="button">Leaderboards</button>
      </nav>
      <BuildStatStrip stats={selectedStats} />
      <section className="build-layout">
        <RunesModule assets={assets} champion={champion} runePage={selectedRunePage} />
        <SummonerSpellsModule assets={assets} champion={champion} spellPair={selectedSpellPair} />
        <DataSourceModule stats={selectedStats} />
        <MatchupsModule />
        <SkillPath champion={champion} />
        <BestBuild champion={champion} />
      </section>
    </section>
  );
}

function ChampionHero({
  abilities,
  champion,
  onBack,
  role,
}: {
  abilities: AbilityInfo[];
  champion: ChampionDetail;
  onBack: () => void;
  role: string;
}): React.JSX.Element {
  return (
    <section
      className="build-hero"
      style={{ "--champion-splash": `url("https://ddragon.leagueoflegends.com/cdn/img/champion/splash/${champion.id}_0.jpg")` } as CSSProperties}
    >
      <Button className="build-back-button" tone="quiet" onClick={onBack}>Retour</Button>
      <div className="build-hero-main">
        <img className="build-portrait" src={champion.iconUrl} alt="" />
        <div className="build-title-block">
          <div className="build-title-row">
            <span className="tier-badge">A</span>
            <h2><span>{champion.name}</span> Build for <span>{roleLabel(role)}</span></h2>
            <span className="patch-badge">Patch {champion.version}</span>
          </div>
          <p>{champion.title}</p>
          <AbilityStrip abilities={abilities} />
        </div>
      </div>
    </section>
  );
}

function useChampionAbilities(champion: ChampionDetail): AbilityInfo[] {
  return useMemo(
    () => [
      {
        description: champion.passive?.description ?? "",
        iconUrl: champion.passive?.image?.full
          ? passiveIconUrl(champion.version, champion.passive.image.full)
          : champion.iconUrl,
        key: "P",
        name: champion.passive?.name ?? "Passive",
      },
      ...champion.spells.map((spell, index) => ({
        cooldown: spell.cooldownBurn,
        cost: spell.costBurn,
        description: abilityDescription(spell),
        iconUrl: spell.image?.full ? spellIconUrl(champion.version, spell.image.full) : champion.iconUrl,
        key: spellKeys[index] ?? "?",
        name: spell.name,
        range: spell.rangeBurn,
      })),
    ],
    [champion],
  );
}

function selectChampionRoleStats(
  stats: ChampionRoleStats[],
  selectedRole: string,
): ChampionRoleStats | undefined {
  const preferredRole = preferredMatchRole(selectedRole);

  return [...stats]
    .sort((first, second) => second.sampleSize - first.sampleSize)
    .find((stat) => preferredRole === undefined || stat.role === preferredRole)
    ?? [...stats].sort((first, second) => second.sampleSize - first.sampleSize)[0];
}

function preferredMatchRole(selectedRole: string): string | undefined {
  const roles: Record<string, string> = {
    Assassin: "MIDDLE",
    Fighter: "TOP",
    Mage: "MIDDLE",
    Marksman: "BOTTOM",
    Support: "UTILITY",
    Tank: "TOP",
  };

  return roles[selectedRole];
}

function formatPercent(value?: number): string {
  if (value === undefined) {
    return "--";
  }

  return `${value.toFixed(1)}%`;
}

function AbilityStrip({ abilities }: { abilities: AbilityInfo[] }): React.JSX.Element {
  const { dispatch, state } = useAppStore();

  return (
    <div className="ability-strip">
      {abilities.map((ability) => (
        <button
          className="ability-chip"
          data-active={state.abilityPanel.abilityKey === ability.key}
          key={ability.key}
          title={ability.name}
          type="button"
          onClick={() => dispatch({ abilityKey: ability.key, type: "abilityPanelToggled" })}
        >
          <img alt="" src={ability.iconUrl} />
          <span>{ability.key}</span>
        </button>
      ))}
    </div>
  );
}

function AbilityPanel({
  ability,
}: {
  ability: AbilityInfo;
}): React.JSX.Element {
  const stats = [
    ["Cooldown", ability.cooldown],
    ["Cost", ability.cost],
    ["Range", ability.range],
  ].filter(([, value]) => value && value !== "0");

  return (
    <div className="ability-popover">
      <span className="ability-key-pill">{ability.key}</span>
      <strong>{ability.name}</strong>
      <p>{ability.description || "Description indisponible dans Data Dragon."}</p>
      <div className="ability-popover-stats">
        {stats.map(([label, value]) => <span key={label}>{label}: {value}</span>)}
      </div>
    </div>
  );
}

function BuildStatStrip({ stats }: { stats?: ChampionRoleStats }): React.JSX.Element {
  return (
    <section className="build-stat-strip">
      {[
        ["Tier", stats?.tier ?? "Local"],
        ["Win Rate", formatPercent(stats?.winRate)],
        ["Rank", stats ? `${stats.wins} / ${stats.sampleSize}` : "-- / --"],
        ["Pick Rate", formatPercent(stats?.pickRate)],
        ["Ban Rate", "--"],
        ["Matches", stats?.sampleSize.toString() ?? "0"],
      ].map(([label, value]) => (
        <article key={label}>
          <span>{label}</span>
          <strong>{value}</strong>
        </article>
      ))}
    </section>
  );
}

function RunesModule({
  assets,
  champion,
  runePage,
}: {
  assets: GameAssets | null;
  champion: ChampionDetail;
  runePage?: ChampionRunePageStats;
}): React.JSX.Element {
  const primary =
    findRuneTreeById(assets, runePage?.primaryStyleId) ??
    assets?.runeTrees.find((tree) => tree.name === "Domination") ??
    assets?.runeTrees[0];
  const secondary =
    findRuneTreeById(assets, runePage?.subStyleId) ??
    assets?.runeTrees.find((tree) => tree.name === "Sorcellerie" || tree.name === "Sorcery") ??
    assets?.runeTrees[1];
  const activeRuneIds = runePage ? new Set(runePage.selectedPerkIds) : undefined;

  return (
    <article className="build-module runes-module">
      <div className="module-header">
        <h3>Recommended</h3>
        <strong>{runePage ? `${runePage.winRate.toFixed(1)}% WR` : "Runes"}</strong>
      </div>
      <div className="rune-board">
        {primary ? (
          <RuneTreeColumn activeRuneIds={activeRuneIds} activeRunes={[0, 4, 8, 9]} tree={primary} />
        ) : (
          <span>Runes a synchroniser</span>
        )}
        {secondary ? (
          <RuneTreeColumn activeRuneIds={activeRuneIds} activeRunes={[1, 5, 8]} compact tree={secondary} />
        ) : null}
      </div>
      <span className="module-footnote">
        {runePage
          ? `${runePage.games} games - ${runePage.wins} wins - ${runePage.source}`
          : `${champion.name} build placeholder until MATCH-V5 aggregates are synced.`}
      </span>
    </article>
  );
}

function findRuneTreeById(assets: GameAssets | null, treeId?: number): RuneTree | undefined {
  if (treeId === undefined) {
    return undefined;
  }

  return assets?.runeTrees.find((tree) => tree.id === treeId);
}

function RuneTreeColumn({
  activeRuneIds,
  activeRunes,
  compact = false,
  tree,
}: {
  activeRuneIds?: Set<number>;
  activeRunes: number[];
  compact?: boolean;
  tree: RuneTree;
}): React.JSX.Element {
  const runes = tree.slots.flatMap((slot) => slot.runes);

  return (
    <div className="rune-column">
      <div className="rune-tree-heading">
        <img src={runeIconUrl(tree.icon)} alt="" />
        <strong>{tree.name}</strong>
      </div>
      <div className={`rune-grid ${compact ? "compact" : ""}`.trim()}>
        {runes.map((rune, index) => (
          <img
            alt={rune.name}
            data-active={activeRuneIds?.has(rune.id) ?? activeRunes.includes(index)}
            key={rune.id}
            src={runeIconUrl(rune.icon)}
            title={rune.name}
          />
        ))}
      </div>
    </div>
  );
}

function SummonerSpellsModule({
  assets,
  champion,
  spellPair,
}: {
  assets: GameAssets | null;
  champion: ChampionDetail;
  spellPair?: ChampionSpellPairStats;
}): React.JSX.Element {
  const spells = spellPair
    ? spellPair.spellIds
        .map((spellId) => findSummonerSpellByKey(assets, spellId))
        .filter((spell): spell is SummonerSpell => spell !== undefined)
    : ["SummonerFlash", "SummonerDot"]
        .map((id) => assets?.summonerSpells[id])
        .filter((spell): spell is SummonerSpell => spell !== undefined);

  return (
    <article className="build-module spells-module">
      <div className="module-header">
        <h3>Summoner Spells</h3>
        <strong>{spellPair ? `${spellPair.winRate.toFixed(1)}% WR` : "-- WR"}</strong>
      </div>
      <div className="spell-pair">
        {spells.map((spell) => (
          <div className="spell-card" key={spell.id}>
            <img
              alt={spell.name}
              src={`https://ddragon.leagueoflegends.com/cdn/${champion.version}/img/spell/${spell.image.full}`}
              title={spell.description}
            />
            <span>{spell.name}</span>
          </div>
        ))}
      </div>
      {spellPair ? (
        <span className="module-footnote">
          {spellPair.games} games - {spellPair.wins} wins - {spellPair.source}
        </span>
      ) : null}
    </article>
  );
}

function findSummonerSpellByKey(
  assets: GameAssets | null,
  spellId: number,
): SummonerSpell | undefined {
  return Object.values(assets?.summonerSpells ?? {}).find((spell) => Number(spell.key) === spellId);
}

function DataSourceModule({ stats }: { stats?: ChampionRoleStats }): React.JSX.Element {
  return (
    <article className="build-module data-source-module">
      <div className="module-header">
        <h3>Stats source</h3>
        <strong>{stats?.source ?? "MATCH-V5"}</strong>
      </div>
      {stats ? (
        <p>
          Echantillon local {stats.platformId}, patch {stats.patch}, role {stats.role}, queue {stats.queueId}.
        </p>
      ) : (
        <p>
          Lance un historique de matchs pour alimenter les aggregats locaux de ce champion.
        </p>
      )}
    </article>
  );
}

function MatchupsModule(): React.JSX.Element {
  return (
    <article className="build-module matchups-module">
      <div className="module-header">
        <h3>Toughest Matchups</h3>
        <span>Ces champions counter le pick</span>
      </div>
      <div className="matchup-list">
        {Array.from({ length: 5 }, (_, index) => (
          <div key={index}>
            <span>?</span>
            <strong>--%</strong>
            <small>0 Games</small>
          </div>
        ))}
      </div>
    </article>
  );
}

function SkillPriorityIcons({ champion }: { champion: ChampionDetail }): React.JSX.Element {
  const primarySpells = champion.spells.slice(0, 3);

  return (
    <div className="skill-priority-inline" aria-label="Skill priority">
      <span>Skill Priority</span>
      <div className="skill-priority-icons">
        {primarySpells.map((spell, index) => (
          <div className="priority-icon" key={spell.name}>
            <img alt="" src={spell.image?.full ? spellIconUrl(champion.version, spell.image.full) : champion.iconUrl} />
            <span>{spellKeys[index]}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function SkillPath({ champion }: { champion: ChampionDetail }): React.JSX.Element {
  const skillPath = ["Q", "E", "W", "Q", "Q", "R", "Q", "W", "Q", "W", "R", "W", "W", "E", "E", "R", "E", "E"];
  const primarySpells = champion.spells.slice(0, 3);

  if (primarySpells.length === 0) {
    return (
      <article className="build-module skill-path-module">
        <div className="module-header">
          <h3>Skill Path</h3>
          <span>Most popular ability leveling order</span>
        </div>
        <span>A synchroniser depuis les timelines MATCH-V5.</span>
      </article>
    );
  }

  return (
    <article className="build-module skill-path-module">
      <div className="module-header">
        <h3>Skill Path</h3>
        <span>Most popular ability leveling order</span>
      </div>
      <div className="skill-order">
        <div className="skill-step skill-step-header">
          <strong />
          <span />
          <div className="skill-levels">
            {skillPath.map((_, index) => <span key={index}>{index + 1}</span>)}
          </div>
        </div>
        {primarySpells.map((spell, index) => {
          const key = spellKeys[index];

          return (
            <div className="skill-step" key={key}>
              <strong>{key}</strong>
              <span>{spell.name}</span>
              <div className="skill-levels">
                {skillPath.map((level, levelIndex) => (
                  <span data-active={level === key} key={`${key}-${levelIndex}`}>
                    {level === key ? levelIndex + 1 : ""}
                  </span>
                ))}
              </div>
            </div>
          );
        })}
      </div>
      <SkillPriorityIcons champion={champion} />
    </article>
  );
}

function BestBuild({ champion }: { champion: ChampionDetail }): React.JSX.Element {
  const items = extractRecommendedBuild(champion.recommended);

  return (
    <article className="build-module items-module">
      <div className="module-header">
        <h3>Best Build</h3>
        <span>Items by phase</span>
      </div>
      <div className="build-list">
        {items.length > 0 ? (
          items.map((item) => (
            <img
              alt={`Item ${item}`}
              key={item}
              src={`https://ddragon.leagueoflegends.com/cdn/${champion.version}/img/item/${item}.png`}
            />
          ))
        ) : (
          <span>A synchroniser depuis les matchs MATCH-V5.</span>
        )}
      </div>
    </article>
  );
}
