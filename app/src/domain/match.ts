export type MatchDetail = {
  durationSeconds: number;
  gameCreatedAt: number;
  matchId: string;
  queueId: number;
  teams: MatchTeam[];
  timeline: MatchTimelinePoint[];
};

export type MatchTeam = {
  participants: MatchParticipant[];
  result: "Defeat" | "Victory";
  teamId: number;
};

export type MatchParticipant = {
  assists: number;
  championId: number;
  championLevel: number;
  championName: string;
  cs: number;
  deaths: number;
  goldEarned: number;
  items: number[];
  kills: number;
  participantId: number;
  riotId: string;
  summonerSpellIds: number[];
  teamPosition: string;
  totalDamageToChampions: number;
  visionScore: number;
  win: boolean;
};

export type MatchTimelinePoint = {
  blueDamage: number;
  blueGold: number;
  blueXp: number;
  minute: number;
  redDamage: number;
  redGold: number;
  redXp: number;
};
