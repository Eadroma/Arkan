import { useEffect, useMemo, useState, type CSSProperties } from "react";

import { itemIconUrl, loadGameAssets, spellIconUrl } from "../../application/dataDragonApi";
import type { SummonerSpell } from "../../domain/assets";
import type { MatchDetail, MatchParticipant, MatchTeam, MatchTimelinePoint } from "../../domain/match";
import { useAppActions } from "../../application/useAppActions";
import { useAppStore } from "../../store/appStore";
import { Button } from "../components/Button";
import { EmptyLines } from "../components/EmptyLines";
import { SurfaceCard } from "../components/SurfaceCard";

export function MatchDetailPage(): React.JSX.Element {
  const { state } = useAppStore();
  const { setView } = useAppActions();

  if (state.matchDetail.status === "loading") {
    return (
      <section className="match-detail-page">
        <SurfaceCard title="Match detail" wide>
          <EmptyLines />
        </SurfaceCard>
      </section>
    );
  }

  if (state.matchDetail.status === "error" || !state.matchDetail.detail) {
    return (
      <section className="match-detail-page">
        <SurfaceCard title="Match indisponible" wide>
          <div className="match-detail-empty">
            <span>Impossible de charger les details du match.</span>
            <Button type="button" onClick={() => void setView("profile")}>Retour</Button>
          </div>
        </SurfaceCard>
      </section>
    );
  }

  return <LoadedMatchDetail detail={state.matchDetail.detail} />;
}

function LoadedMatchDetail({ detail }: { detail: MatchDetail }): React.JSX.Element {
  const { setView } = useAppActions();
  const { state } = useAppStore();
  const championIconByKey = useMemo(
    () => new Map(state.championCatalog.map((champion) => [champion.key, champion.iconUrl])),
    [state.championCatalog],
  );
  const version = state.championCatalog[0]?.version;
  const spellByKey = useSummonerSpellByKey(version);

  return (
    <section className="match-detail-page">
      <section className="match-detail-hero">
        <Button type="button" tone="quiet" onClick={() => void setView("profile")}>Retour</Button>
        <div>
          <p className="panel-kicker">{queueLabel(detail.queueId)} - {formatDuration(detail.durationSeconds)}</p>
          <h2>{detail.matchId}</h2>
        </div>
      </section>

      <MatchGraph timeline={detail.timeline} />

      <section className="match-scoreboard">
        {detail.teams.map((team) => (
          <TeamTable
            championIconByKey={championIconByKey}
            itemVersion={version}
            key={team.teamId}
            spellByKey={spellByKey}
            team={team}
          />
        ))}
      </section>
    </section>
  );
}

function TeamTable({
  championIconByKey,
  itemVersion,
  spellByKey,
  team,
}: {
  championIconByKey: Map<string, string>;
  itemVersion?: string;
  spellByKey: Map<number, SummonerSpell>;
  team: MatchTeam;
}): React.JSX.Element {
  const teamName = team.teamId === 100 ? "Blue Team" : "Red Team";

  return (
    <section className="team-table" data-result={team.result.toLowerCase()}>
      <div className="team-table__header">
        <strong>{team.result} ({teamName})</strong>
        <span>Carry</span>
        <span>KDA</span>
        <span>Damage</span>
        <span>Gold</span>
        <span>CS</span>
        <span>Wards</span>
        <span>Items</span>
      </div>
      {team.participants.map((participant) => (
        <ParticipantRow
          championIconUrl={championIconByKey.get(participant.championId.toString())}
          itemVersion={itemVersion}
          key={participant.participantId}
          participant={participant}
          spellByKey={spellByKey}
        />
      ))}
    </section>
  );
}

function ParticipantRow({
  championIconUrl,
  itemVersion,
  participant,
  spellByKey,
}: {
  championIconUrl?: string;
  itemVersion?: string;
  participant: MatchParticipant;
  spellByKey: Map<number, SummonerSpell>;
}): React.JSX.Element {
  const { openPlayerProfile } = useAppActions();

  return (
    <div className="participant-row">
      <button
        className="participant-row__identity"
        disabled={!participant.riotId.includes("#")}
        type="button"
        onClick={() => void openPlayerProfile(participant.riotId)}
      >
        {championIconUrl ? <img src={championIconUrl} alt="" /> : <span className="participant-row__fallback">?</span>}
        <span className="participant-row__level">{participant.championLevel}</span>
        <div className="participant-spells">
          {participant.summonerSpellIds.map((spellId) => {
            const spell = spellByKey.get(spellId);

            return spell && itemVersion ? (
              <img
                alt={spell.name}
                key={spellId}
                src={spellIconUrl(itemVersion, spell.image.full)}
                title={spell.description}
              />
            ) : (
              <span key={spellId}>{spellId}</span>
            );
          })}
        </div>
        <div>
          <strong>{participant.riotId}</strong>
          <span>{participant.teamPosition}</span>
        </div>
      </button>
      <strong>{carryScore(participant)}</strong>
      <div>
        <strong>{participant.kills} / {participant.deaths} / {participant.assists}</strong>
        <span>{kda(participant)} KDA</span>
      </div>
      <DamageCell value={participant.totalDamageToChampions} />
      <strong>{formatCompact(participant.goldEarned)}</strong>
      <strong>{participant.cs}</strong>
      <strong>{participant.visionScore}</strong>
      <div className="participant-items">
        {participant.items.map((itemId) =>
          itemVersion ? <img src={itemIconUrl(itemVersion, itemId)} alt="" key={itemId} /> : <span key={itemId} />,
        )}
      </div>
    </div>
  );
}

function useSummonerSpellByKey(version?: string): Map<number, SummonerSpell> {
  const [spells, setSpells] = useState<Map<number, SummonerSpell>>(new Map());

  useEffect(() => {
    if (!version) {
      return;
    }

    let isMounted = true;

    void loadGameAssets(version).then((assets) => {
      if (!isMounted) {
        return;
      }

      setSpells(
        new Map(
          Object.values(assets.summonerSpells).map((spell) => [Number(spell.key), spell]),
        ),
      );
    });

    return () => {
      isMounted = false;
    };
  }, [version]);

  return spells;
}

function DamageCell({ value }: { value: number }): React.JSX.Element {
  const width = Math.min(100, Math.max(8, value / 400));

  return (
    <div className="damage-cell">
      <strong>{formatCompact(value)}</strong>
      <span style={{ inlineSize: `${width}%` }} />
    </div>
  );
}

function MatchGraph({ timeline }: { timeline: MatchTimelinePoint[] }): React.JSX.Element {
  const lastPoint = timeline[timeline.length - 1];

  return (
    <SurfaceCard title="Timeline" aside={<span className="muted">Gold / XP / Damage</span>} wide>
      {timeline.length === 0 || !lastPoint ? (
        <span className="match-history-state">Timeline indisponible pour ce match.</span>
      ) : (
        <div className="timeline-graphs">
          <TimelineGraph
            blueValue={lastPoint.blueGold}
            label="Gold"
            redValue={lastPoint.redGold}
            timeline={timeline}
            valueKeyBlue="blueGold"
            valueKeyRed="redGold"
          />
          <TimelineGraph
            blueValue={lastPoint.blueXp}
            label="XP"
            redValue={lastPoint.redXp}
            timeline={timeline}
            valueKeyBlue="blueXp"
            valueKeyRed="redXp"
          />
          <TimelineGraph
            blueValue={lastPoint.blueDamage}
            label="Damage"
            redValue={lastPoint.redDamage}
            timeline={timeline}
            valueKeyBlue="blueDamage"
            valueKeyRed="redDamage"
          />
        </div>
      )}
    </SurfaceCard>
  );
}

function TimelineGraph({
  blueValue,
  label,
  redValue,
  timeline,
  valueKeyBlue,
  valueKeyRed,
}: {
  blueValue: number;
  label: string;
  redValue: number;
  timeline: MatchTimelinePoint[];
  valueKeyBlue: keyof MatchTimelinePoint;
  valueKeyRed: keyof MatchTimelinePoint;
}): React.JSX.Element {
  const maxValue = Math.max(...timeline.flatMap((point) => [Number(point[valueKeyBlue]), Number(point[valueKeyRed])]), 1);

  return (
    <article className="timeline-graph">
      <div>
        <strong>{label}</strong>
        <span>Blue {formatCompact(blueValue)} / Red {formatCompact(redValue)}</span>
      </div>
      <div className="timeline-bars">
        {timeline.map((point) => (
          <span
            key={`${label}-${point.minute}`}
            style={{
              "--blue": `${(Number(point[valueKeyBlue]) / maxValue) * 100}%`,
              "--red": `${(Number(point[valueKeyRed]) / maxValue) * 100}%`,
            } as CSSProperties}
            title={`${point.minute} min`}
          />
        ))}
      </div>
    </article>
  );
}

function carryScore(participant: MatchParticipant): number {
  return Math.round((participant.kills * 2 + participant.assists + participant.cs / 20 + participant.visionScore / 3));
}

function kda(participant: MatchParticipant): string {
  const ratio = (participant.kills + participant.assists) / Math.max(1, participant.deaths);

  return ratio.toFixed(2);
}

function formatCompact(value: number): string {
  return new Intl.NumberFormat("en", {
    maximumFractionDigits: 1,
    notation: "compact",
  }).format(value);
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
