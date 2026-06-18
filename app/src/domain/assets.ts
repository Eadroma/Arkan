export type RuneTree = {
  icon: string;
  id: number;
  key: string;
  name: string;
  slots: Array<{
    runes: Array<{
      icon: string;
      id: number;
      key: string;
      name: string;
    }>;
  }>;
};

export type SummonerSpell = {
  description: string;
  id: string;
  image: {
    full: string;
  };
  name: string;
};

export type GameAssets = {
  runeTrees: RuneTree[];
  summonerSpells: Record<string, SummonerSpell>;
};
