import { extractRecommendedBuild, filterChampions, roleLabel, type ChampionSummary } from "./champion-model";

type ViewName = "profile" | "champions" | "champion-detail";
type StateVariant = "error" | "loading" | "offline" | "online" | "warning";

type LeagueClientState = {
  variant: StateVariant;
  title: string;
  kicker: string;
  pill: string;
  status: string;
  region: string;
  level: string | number;
};

type ChampionMastery = {
  championId?: number;
  championLevel?: number;
  championPoints?: number;
};

type RiotAccount = {
  gameName: string;
  tagLine: string;
  profileIconId?: number;
  summonerLevel?: number;
  championMasteries?: ChampionMastery[];
};

type LeagueSummoner = {
  gameName?: string;
  tagLine?: string;
  displayName: string;
  profileIconId?: number;
  summonerLevel?: number;
  championMasteries?: ChampionMastery[];
};

type LeagueClientStatus = {
  cached: boolean;
  connected: boolean;
  detected: boolean;
  summoner?: LeagueSummoner;
};

type TauriInvoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

declare global {
  interface Window {
    __TAURI__?: {
      core?: {
        invoke: TauriInvoke;
      };
    };
  }
}

type DataDragonChampion = {
  id: string;
  key: string;
  name: string;
  title: string;
  tags: ChampionSummary["tags"];
  image: {
    full: string;
  };
  spells?: Array<{ name: string }>;
  recommended?: Array<{ blocks?: Array<{ items?: Array<{ id: string }> }> }>;
};

type ChampionDetail = ChampionSummary & {
  spells: Array<{ name: string }>;
  recommended: Array<{ blocks?: Array<{ items?: Array<{ id: string }> }> }>;
};

const requiredElement = <T extends Element>(selector: string, root: ParentNode = document): T => {
  const element = root.querySelector<T>(selector);

  if (!element) {
    throw new Error(`Missing required element: ${selector}`);
  }

  return element;
};

const shell = requiredElement<HTMLElement>(".shell");
const toggle = requiredElement<HTMLButtonElement>(".sidebar-toggle");
const playerTitle = requiredElement<HTMLElement>("[data-player-title]");
const lcuKicker = requiredElement<HTMLElement>("[data-lcu-kicker]");
const lcuPill = requiredElement<HTMLElement>("[data-lcu-pill]");
const lcuStatus = requiredElement<HTMLElement>("[data-lcu-status]");
const lcuRegion = requiredElement<HTMLElement>("[data-lcu-region]");
const lcuLevel = requiredElement<HTMLElement>("[data-lcu-level]");
const avatarImg = requiredElement<HTMLImageElement>("[data-avatar-img]");
const avatarFallback = requiredElement<HTMLElement>("[data-avatar-fallback]");
const brandHome = requiredElement<HTMLAnchorElement>("[data-brand-home]");
const searchForm = requiredElement<HTMLFormElement>("[data-search-form]");
const searchInput = requiredElement<HTMLInputElement>("[data-search-input]");
const searchRegion = requiredElement<HTMLSelectElement>("[data-search-region]");
const searchButton = requiredElement<HTMLButtonElement>("[data-search-button]");
const championPool = requiredElement<HTMLElement>("[data-champion-pool]");
const championEmpty = requiredElement<HTMLElement>("[data-champion-empty]");
const viewTitle = requiredElement<HTMLElement>("[data-view-title]");
const profileToolbar = requiredElement<HTMLElement>("[data-profile-toolbar]");
const navItems = [...document.querySelectorAll<HTMLElement>("[data-nav-view]")];
const viewPanels = [...document.querySelectorAll<HTMLElement>("[data-view-panel]")];
const championSearch = requiredElement<HTMLInputElement>("[data-champion-search]");
const championRole = requiredElement<HTMLSelectElement>("[data-champion-role]");
const championCatalog = requiredElement<HTMLElement>("[data-champion-catalog]");
const championBack = requiredElement<HTMLButtonElement>("[data-champion-back]");
const championDetailIcon = requiredElement<HTMLImageElement>("[data-champion-detail-icon]");
const championDetailRole = requiredElement<HTMLElement>("[data-champion-detail-role]");
const championDetailName = requiredElement<HTMLElement>("[data-champion-detail-name]");
const championDetailTitle = requiredElement<HTMLElement>("[data-champion-detail-title]");
const championDetailRoleSelect = requiredElement<HTMLSelectElement>("[data-champion-detail-role-select]");
const championWinrate = requiredElement<HTMLElement>("[data-champion-winrate]");
const championPickrate = requiredElement<HTMLElement>("[data-champion-pickrate]");
const championSample = requiredElement<HTMLElement>("[data-champion-sample]");
const championSkillOrder = requiredElement<HTMLElement>("[data-champion-skill-order]");
const championBuild = requiredElement<HTMLElement>("[data-champion-build]");

let championCatalogItems: ChampionDetail[] = [];
let selectedChampion: ChampionDetail | null = null;

const savedSidebarState = localStorage.getItem("arkan.sidebar");

if (savedSidebarState === "collapsed") {
  shell.dataset.sidebar = "collapsed";
  toggle.setAttribute("aria-label", "Expand sidebar");
}

toggle.addEventListener("click", () => {
  const collapsed = shell.dataset.sidebar === "collapsed";
  shell.dataset.sidebar = collapsed ? "expanded" : "collapsed";
  localStorage.setItem("arkan.sidebar", collapsed ? "expanded" : "collapsed");
  toggle.setAttribute("aria-label", collapsed ? "Reduce sidebar" : "Expand sidebar");
});

brandHome.addEventListener("click", handleConnectedPlayerReset);
searchForm.addEventListener("submit", handleManualSearch);
championSearch.addEventListener("input", () => renderChampionCatalog(championSearch.value));
championRole.addEventListener("change", () => renderChampionCatalog(championSearch.value));
championBack.addEventListener("click", () => setActiveView("champions"));
championDetailRoleSelect.addEventListener("change", () => {
  if (selectedChampion) {
    renderChampionDetail(selectedChampion, championDetailRoleSelect.value);
  }
});

navItems.forEach((item) => {
  item.addEventListener("click", (event) => {
    event.preventDefault();
    setActiveView(item.dataset.navView as ViewName);
  });
});

detectLeagueClient();

async function handleConnectedPlayerReset(event: Event): Promise<void> {
  event.preventDefault();
  setActiveView("profile");
  searchInput.value = "";
  setSearchPending(false);
  setLeagueClientState({
    variant: "loading",
    title: "Recherche automatique...",
    kicker: "Retour profil connecte",
    pill: "Scanning",
    status: "Checking",
    region: "EUW1",
    level: "--",
  });
  resetProfileIcon();
  resetChampionPool();
  await detectLeagueClient();
}

async function handleManualSearch(event: Event): Promise<void> {
  event.preventDefault();
  const invoke = window.__TAURI__?.core?.invoke;
  const rawRiotId = searchInput.value.trim();
  const region = searchRegion.value;

  if (!rawRiotId) {
    setLeagueClientState({
      variant: "warning",
      title: "Riot ID requis",
      kicker: "Recherche manuelle",
      pill: "Input",
      status: "Missing",
      region,
      level: "--",
    });
    resetProfileIcon();
    resetChampionPool();
    return;
  }

  setSearchPending(true);
  setLeagueClientState({
    variant: "loading",
    title: rawRiotId,
    kicker: "Recherche Riot ID",
    pill: "Searching",
    status: "Lookup",
    region,
    level: "--",
  });
  resetProfileIcon();
  resetChampionPool();

  if (!invoke) {
    setSearchPending(false);
    setLeagueClientState({
      variant: "warning",
      title: rawRiotId,
      kicker: "Mode preview",
      pill: "Preview",
      status: "Local",
      region,
      level: "--",
    });
    return;
  }

  try {
    const normalizedRiotId = await invoke<string>("parse_riot_id", { input: rawRiotId });
    const account = await invoke<RiotAccount>("resolve_riot_account", {
      input: normalizedRiotId,
      platform: region,
    });

    setLeagueClientState({
      variant: "online",
      title: `${account.gameName}#${account.tagLine}`,
      kicker: "Compte Riot resolu",
      pill: "Resolved",
      status: "Resolved",
      region,
      level: account.summonerLevel ?? "--",
    });
    setProfileIcon(account.profileIconId);
    setChampionPool(account.championMasteries);
  } catch (error) {
    setLeagueClientState({
      variant: "warning",
      title: rawRiotId,
      kicker: friendlySearchError(error),
      pill: "Config",
      status: "Needs key",
      region,
      level: "--",
    });
    resetChampionPool();
  } finally {
    setSearchPending(false);
  }
}

async function detectLeagueClient(): Promise<void> {
  const invoke = window.__TAURI__?.core?.invoke;

  if (!invoke) {
    setLeagueClientState({
      variant: "offline",
      title: "GameName#TAG",
      kicker: "Mode preview",
      pill: "Preview",
      status: "Preview",
      region: "EUW1",
      level: "--",
    });
    return;
  }

  try {
    const status = await invoke<LeagueClientStatus>("league_client_status");

    if (status.connected && status.summoner) {
      const summoner = status.summoner;
      const riotId =
        summoner.gameName && summoner.tagLine
          ? `${summoner.gameName}#${summoner.tagLine}`
          : summoner.displayName;
      const cacheLabel = status.cached ? "Synced" : "Detected";

      setLeagueClientState({
        variant: "online",
        title: riotId,
        kicker: "Joueur connecte detecte",
        pill: cacheLabel,
        status: cacheLabel,
        region: "EUW1",
        level: summoner.summonerLevel ?? "--",
      });
      setProfileIcon(summoner.profileIconId);
      setChampionPool(summoner.championMasteries);
      hydrateConnectedProfileFromRiotId(invoke, riotId);
      return;
    }

    setLeagueClientState({
      variant: status.detected ? "warning" : "offline",
      title: status.detected ? "Client detecte" : "GameName#TAG",
      kicker: status.detected ? "Connexion locale incomplete" : "Aucun client League",
      pill: status.detected ? "Detected" : "Offline",
      status: status.detected ? "Partial" : "Offline",
      region: "EUW1",
      level: "--",
    });
    resetChampionPool();
  } catch {
    setLeagueClientState({
      variant: "warning",
      title: "Detection indisponible",
      kicker: "Erreur locale",
      pill: "Error",
      status: "Error",
      region: "EUW1",
      level: "--",
    });
    resetProfileIcon();
    resetChampionPool();
  }
}

function setLeagueClientState({ variant, title, kicker, pill, status, region, level }: LeagueClientState): void {
  playerTitle.textContent = title;
  lcuKicker.textContent = kicker;
  lcuPill.textContent = pill;
  lcuPill.dataset.state = variant;
  lcuStatus.textContent = status;
  lcuRegion.textContent = region;
  lcuLevel.textContent = String(level);
}

function setSearchPending(isPending: boolean): void {
  searchInput.disabled = isPending;
  searchRegion.disabled = isPending;
  searchButton.disabled = isPending;
  searchButton.textContent = isPending ? "..." : "Search";
}

function friendlySearchError(error: unknown): string {
  const message = String(error ?? "Erreur recherche");

  if (message.toLowerCase().includes("riot api key")) {
    return "Cle Riot API manquante";
  }

  if (message.toLowerCase().includes("separator")) {
    return "Format attendu GameName#TAG";
  }

  return message;
}

async function hydrateConnectedProfileFromRiotId(invoke: TauriInvoke, riotId: string): Promise<void> {
  if (!riotId.includes("#")) {
    return;
  }

  try {
    const account = await invoke<RiotAccount>("resolve_riot_account", {
      input: riotId,
      platform: "EUW1",
    });

    setProfileIcon(account.profileIconId);

    if (account.summonerLevel) {
      lcuLevel.textContent = String(account.summonerLevel);
    }

    setChampionPool(account.championMasteries);
  } catch {
    // The LCU profile remains useful even if Riot API enrichment fails.
  }
}

async function setProfileIcon(profileIconId?: number): Promise<void> {
  if (!profileIconId) {
    resetProfileIcon();
    return;
  }

  try {
    const response = await fetch("https://ddragon.leagueoflegends.com/api/versions.json");
    const versions = (await response.json()) as string[];
    const version = versions[0];
    const iconUrl = `https://ddragon.leagueoflegends.com/cdn/${version}/img/profileicon/${profileIconId}.png`;

    avatarImg.onload = () => {
      avatarImg.hidden = false;
      avatarFallback.hidden = true;
    };
    avatarImg.onerror = resetProfileIcon;
    avatarImg.src = iconUrl;
  } catch {
    resetProfileIcon();
  }
}

function resetProfileIcon(): void {
  avatarImg.removeAttribute("src");
  avatarImg.hidden = true;
  avatarFallback.hidden = false;
}

async function setChampionPool(masteries?: ChampionMastery[]): Promise<void> {
  if (!Array.isArray(masteries) || masteries.length === 0) {
    resetChampionPool();
    return;
  }

  try {
    const championIndex = await loadChampionIndex();

    championPool.replaceChildren(
      ...masteries.slice(0, 5).map((mastery) => {
        const champion = championIndex.get(String(mastery.championId));
        const row = document.createElement("button");
        const icon = document.createElement("img");
        const body = document.createElement("div");
        const name = document.createElement("strong");
        const meta = document.createElement("span");
        const points = new Intl.NumberFormat("fr-FR").format(mastery.championPoints ?? 0);

        row.className = "champion-row";
        row.type = "button";
        row.disabled = !champion;

        if (champion) {
          row.addEventListener("click", () => openChampionDetail(champion.id));
        }

        icon.alt = "";
        icon.src = champion?.iconUrl ?? "";
        icon.hidden = !champion;
        name.textContent = champion?.name ?? `Champion ${mastery.championId}`;
        meta.textContent = `M${mastery.championLevel ?? "--"} - ${points} pts`;
        body.append(name, meta);
        row.append(icon, body);
        return row;
      }),
    );

    championPool.hidden = false;
    championEmpty.hidden = true;
  } catch {
    resetChampionPool();
  }
}

function resetChampionPool(): void {
  championPool.replaceChildren();
  championPool.hidden = true;
  championEmpty.hidden = false;
}

async function loadChampionIndex(): Promise<Map<string, ChampionDetail>> {
  const response = await fetch("https://ddragon.leagueoflegends.com/api/versions.json");
  const versions = (await response.json()) as string[];
  const version = versions[0];
  const championResponse = await fetch(
    `https://ddragon.leagueoflegends.com/cdn/${version}/data/fr_FR/champion.json`,
  );
  const championData = (await championResponse.json()) as { data: Record<string, DataDragonChampion> };

  return new Map(
    Object.values(championData.data).map((champion) => [
      champion.key,
      {
        id: champion.id,
        key: champion.key,
        name: champion.name,
        title: champion.title,
        tags: champion.tags,
        spells: champion.spells ?? [],
        recommended: champion.recommended ?? [],
        iconUrl: `https://ddragon.leagueoflegends.com/cdn/${version}/img/champion/${champion.image.full}`,
        version,
      },
    ]),
  );
}

async function setActiveView(viewName: ViewName): Promise<void> {
  navItems.forEach((item) => {
    const isActive = item.dataset.navView === viewName || (viewName === "champion-detail" && item.dataset.navView === "champions");
    item.classList.toggle("active", isActive);
    item.toggleAttribute("aria-current", isActive);
  });

  viewPanels.forEach((panel) => {
    panel.hidden = panel.dataset.viewPanel !== viewName;
  });

  profileToolbar.hidden = viewName !== "profile";
  viewTitle.textContent = viewName === "champions" || viewName === "champion-detail" ? "Champions" : "Profil joueur";

  if (viewName === "champions" || viewName === "champion-detail") {
    await loadChampionCatalog();
  }
}

async function loadChampionCatalog(): Promise<void> {
  if (championCatalogItems.length > 0) {
    renderChampionCatalog(championSearch.value);
    return;
  }

  const championIndex = await loadChampionIndex();
  championCatalogItems = [...championIndex.values()].sort((first, second) => first.name.localeCompare(second.name, "fr"));
  renderChampionCatalog(championSearch.value);
}

function renderChampionCatalog(query = ""): void {
  const selectedRole = championRole.value as ChampionSummary["tags"][number] | "all";
  const champions = filterChampions(championCatalogItems, query, selectedRole);

  championCatalog.replaceChildren(
    ...champions.map((champion) => {
      const item = document.createElement("article");
      const button = document.createElement("button");
      const icon = document.createElement("img");
      const title = document.createElement("strong");
      const subtitle = document.createElement("span");

      item.className = "champion-tile-wrap";
      button.className = "champion-tile";
      button.type = "button";
      button.addEventListener("click", () => openChampionDetail(champion.id));
      icon.alt = "";
      icon.src = champion.iconUrl;
      title.textContent = champion.name;
      subtitle.textContent = champion.title;
      button.append(icon, title, subtitle);
      item.append(button);
      return item;
    }),
  );
}

async function openChampionDetail(championId: string): Promise<void> {
  const champion = await loadChampionDetail(championId);
  selectedChampion = champion;
  renderChampionDetail(champion, champion.tags[0] ?? "ALL");
  await setActiveView("champion-detail");
}

async function loadChampionDetail(championId: string): Promise<ChampionDetail> {
  const champion = championCatalogItems.find((item) => item.id === championId);

  if (!champion) {
    throw new Error(`Champion inconnu: ${championId}`);
  }

  if (champion.spells.length > 0 || champion.recommended.length > 0) {
    return champion;
  }

  const response = await fetch(
    `https://ddragon.leagueoflegends.com/cdn/${champion.version}/data/fr_FR/champion/${champion.id}.json`,
  );
  const details = (await response.json()) as { data: Record<string, DataDragonChampion> };
  const detailedChampion = details.data[champion.id];
  const hydratedChampion: ChampionDetail = {
    ...champion,
    spells: detailedChampion.spells ?? [],
    recommended: detailedChampion.recommended ?? [],
  };

  championCatalogItems = championCatalogItems.map((item) => (item.id === champion.id ? hydratedChampion : item));
  return hydratedChampion;
}

function renderChampionDetail(champion: ChampionDetail, role: string): void {
  championDetailIcon.src = champion.iconUrl;
  championDetailName.textContent = champion.name;
  championDetailTitle.textContent = champion.title;
  championDetailRole.textContent = roleLabel(role);
  championWinrate.textContent = "--";
  championPickrate.textContent = "--";
  championSample.textContent = "0";

  championDetailRoleSelect.replaceChildren(
    ...champion.tags.map((tag) => {
      const option = document.createElement("option");
      option.value = tag;
      option.textContent = roleLabel(tag);
      option.selected = tag === role;
      return option;
    }),
  );

  renderSkillOrder(champion);
  renderBuild(champion);
}

function renderSkillOrder(champion: ChampionDetail): void {
  const primarySpells = champion.spells.slice(0, 3);

  championSkillOrder.replaceChildren(
    ...primarySpells.map((spell, index) => {
      const item = document.createElement("div");
      const key = document.createElement("strong");
      const name = document.createElement("span");

      item.className = "skill-step";
      key.textContent = ["Q", "W", "E"][index] ?? "?";
      name.textContent = spell.name;
      item.append(key, name);
      return item;
    }),
  );

  if (primarySpells.length === 0) {
    championSkillOrder.textContent = "A synchroniser depuis les timelines MATCH-V5.";
  }
}

function renderBuild(champion: ChampionDetail): void {
  const build = extractRecommendedBuild(champion.recommended);

  championBuild.replaceChildren(
    ...build.map((itemId) => {
      const item = document.createElement("img");
      item.alt = `Item ${itemId}`;
      item.src = `https://ddragon.leagueoflegends.com/cdn/${champion.version}/img/item/${itemId}.png`;
      return item;
    }),
  );

  if (build.length === 0) {
    championBuild.textContent = "A synchroniser depuis les matchs MATCH-V5.";
  }
}
