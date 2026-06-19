export type ChampionMastery = {
  championId?: number;
  championLevel?: number;
  championPoints?: number;
};

export type RiotAccount = {
  championMasteries?: ChampionMastery[];
  gameName: string;
  profileIconId?: number;
  puuid?: string;
  summonerLevel?: number;
  tagLine: string;
};

export type LeagueSummoner = {
  championMasteries?: ChampionMastery[];
  displayName: string;
  gameName?: string;
  profileIconId?: number;
  summonerLevel?: number;
  tagLine?: string;
};

export type LeagueClientStatus = {
  cached: boolean;
  connected: boolean;
  detected: boolean;
  summoner?: LeagueSummoner;
};

export type PlayerProfile = {
  championMasteries: ChampionMastery[];
  displayName: string;
  iconUrl?: string;
  kicker: string;
  level: string;
  region: string;
  status: string;
};

export type MatchHistoryEntry = {
  assists: number;
  championId: number;
  championName: string;
  deaths: number;
  durationSeconds: number;
  gameCreatedAt: number;
  kills: number;
  lpDelta: number | null;
  matchId: string;
  queueId: number;
  role: string;
  win: boolean;
};

export type LeagueClientCard = {
  level: string;
  pill: string;
  region: string;
  status: string;
  variant: "error" | "loading" | "offline" | "online" | "warning";
};
