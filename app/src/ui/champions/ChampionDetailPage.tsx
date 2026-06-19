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
  type ChampionSpell,
} from "../../domain/champion";
import { useAppActions } from "../../application/useAppActions";
import { useAppStore } from "../../store/appStore";
import { Button } from "../components/Button";

const spellKeys = ["Q", "W", "E", "R"] as const;

export function ChampionDetailPage({ champion }: { champion: ChampionDetail }): React.JSX.Element {
  const { dispatch, state } = useAppStore();
  const { setView } = useAppActions();
  const [assets, setAssets] = useState<GameAssets | null>(null);
  const role = state.selectedChampionRole;

  useEffect(() => {
    void loadGameAssets(champion.version).then(setAssets);
  }, [champion.version]);

  return (
    <section className="dashboard champion-build-page">
      <ChampionHero champion={champion} role={role} onBack={() => void setView("champions")} />
      <nav className="build-tabs" aria-label="Champion build tabs">
        <button className="active" type="button">Build</button>
        <button type="button">Counters</button>
        <button type="button">Leaderboards</button>
      </nav>
      <section className="build-filter-bar">
        <select
          aria-label="Role champion"
          value={role}
          onChange={(event) =>
            dispatch({ role: event.currentTarget.value, type: "selectedChampionRoleChanged" })
          }
        >
          {champion.tags.map((tag) => (
            <option key={tag} value={tag}>{roleLabel(tag)}</option>
          ))}
        </select>
        <select aria-label="Tier" defaultValue="Emerald +">
          <option>Emerald +</option>
        </select>
        <select aria-label="Queue" defaultValue="Ranked Solo/Duo">
          <option>Ranked Solo/Duo</option>
        </select>
        <Button tone="quiet">More...</Button>
      </section>
      <BuildStatStrip />
      <section className="build-layout">
        <RunesModule assets={assets} champion={champion} />
        <SummonerSpellsModule assets={assets} champion={champion} />
        <DataSourceModule />
        <MatchupsModule />
        <SkillPriority champion={champion} />
        <SkillPath champion={champion} />
        <BestBuild champion={champion} />
      </section>
    </section>
  );
}

function ChampionHero({
  champion,
  onBack,
  role,
}: {
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
          <AbilityStrip champion={champion} />
        </div>
      </div>
    </section>
  );
}

function AbilityStrip({ champion }: { champion: ChampionDetail }): React.JSX.Element {
  const { dispatch, state } = useAppStore();
  const abilities = useMemo(
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
  const selectedAbility = abilities.find((ability) => ability.key === state.abilityPanel.abilityKey);

  return (
    <>
      <div className="ability-strip">
        {abilities.map((ability) => (
          <button
            className="ability-chip"
            data-active={selectedAbility?.key === ability.key}
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
      {selectedAbility ? <AbilityPanel ability={selectedAbility} /> : null}
    </>
  );
}

function AbilityPanel({
  ability,
}: {
  ability: { cooldown?: string; cost?: string; description: string; key: string; name: string; range?: string };
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

function BuildStatStrip(): React.JSX.Element {
  return (
    <section className="build-stat-strip">
      {[
        ["Tier", "A"],
        ["Win Rate", "--"],
        ["Rank", "-- / --"],
        ["Pick Rate", "--"],
        ["Ban Rate", "--"],
        ["Matches", "0"],
      ].map(([label, value]) => (
        <article key={label}>
          <span>{label}</span>
          <strong>{value}</strong>
        </article>
      ))}
    </section>
  );
}

function RunesModule({ assets, champion }: { assets: GameAssets | null; champion: ChampionDetail }): React.JSX.Element {
  const primary = assets?.runeTrees.find((tree) => tree.name === "Domination") ?? assets?.runeTrees[0];
  const secondary =
    assets?.runeTrees.find((tree) => tree.name === "Sorcellerie" || tree.name === "Sorcery") ??
    assets?.runeTrees[1];

  return (
    <article className="build-module runes-module">
      <div className="module-header">
        <h3>Recommended</h3>
        <strong>Runes</strong>
      </div>
      <div className="rune-board">
        {primary ? <RuneTreeColumn activeRunes={[0, 4, 8, 9]} tree={primary} /> : <span>Runes a synchroniser</span>}
        {secondary ? <RuneTreeColumn activeRunes={[1, 5, 8]} compact tree={secondary} /> : null}
      </div>
      <span className="module-footnote">{champion.name} build placeholder until MATCH-V5 aggregates are synced.</span>
    </article>
  );
}

function RuneTreeColumn({
  activeRunes,
  compact = false,
  tree,
}: {
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
            data-active={activeRunes.includes(index)}
            key={rune.id}
            src={runeIconUrl(rune.icon)}
            title={rune.name}
          />
        ))}
      </div>
    </div>
  );
}

function SummonerSpellsModule({ assets, champion }: { assets: GameAssets | null; champion: ChampionDetail }): React.JSX.Element {
  const spells = ["SummonerFlash", "SummonerDot"]
    .map((id) => assets?.summonerSpells[id])
    .filter((spell): spell is SummonerSpell => spell !== undefined);

  return (
    <article className="build-module spells-module">
      <div className="module-header">
        <h3>Summoner Spells</h3>
        <strong>-- WR</strong>
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
    </article>
  );
}

function DataSourceModule(): React.JSX.Element {
  return (
    <article className="build-module data-source-module">
      <div className="module-header">
        <h3>Stats source</h3>
        <strong>MATCH-V5</strong>
      </div>
      <p>
        Les builds publics seront branches depuis des aggregats de matchs par champion, role, rang et patch.
      </p>
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

function SkillPriority({ champion }: { champion: ChampionDetail }): React.JSX.Element {
  const primarySpells = champion.spells.slice(0, 3);

  return (
    <article className="build-module skill-priority-module">
      <div className="module-header">
        <h3>Skill Priority</h3>
        <strong>-- WR</strong>
      </div>
      <div className="skill-priority-icons">
        {primarySpells.map((spell, index) => (
          <div className="priority-icon" key={spell.name}>
            <img alt="" src={spell.image?.full ? spellIconUrl(champion.version, spell.image.full) : champion.iconUrl} />
            <span>{spellKeys[index]}</span>
          </div>
        ))}
      </div>
    </article>
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
