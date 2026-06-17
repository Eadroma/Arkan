const shell = document.querySelector(".shell");
const toggle = document.querySelector(".sidebar-toggle");
const playerTitle = document.querySelector("[data-player-title]");
const lcuKicker = document.querySelector("[data-lcu-kicker]");
const lcuPill = document.querySelector("[data-lcu-pill]");
const lcuStatus = document.querySelector("[data-lcu-status]");
const lcuRegion = document.querySelector("[data-lcu-region]");
const lcuLevel = document.querySelector("[data-lcu-level]");
const avatarImg = document.querySelector("[data-avatar-img]");
const avatarFallback = document.querySelector("[data-avatar-fallback]");
const brandHome = document.querySelector("[data-brand-home]");
const searchForm = document.querySelector("[data-search-form]");
const searchInput = document.querySelector("[data-search-input]");
const searchRegion = document.querySelector("[data-search-region]");
const searchButton = document.querySelector("[data-search-button]");
const championPool = document.querySelector("[data-champion-pool]");
const championEmpty = document.querySelector("[data-champion-empty]");
const viewTitle = document.querySelector("[data-view-title]");
const profileToolbar = document.querySelector("[data-profile-toolbar]");
const navItems = [...document.querySelectorAll("[data-nav-view]")];
const viewPanels = [...document.querySelectorAll("[data-view-panel]")];
const championSearch = document.querySelector("[data-champion-search]");
const championCatalog = document.querySelector("[data-champion-catalog]");
let championCatalogItems = [];

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
navItems.forEach((item) => {
  item.addEventListener("click", (event) => {
    event.preventDefault();
    setActiveView(item.dataset.navView);
  });
});

detectLeagueClient();

async function handleConnectedPlayerReset(event) {
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

async function handleManualSearch(event) {
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
    resetChampionPool();
    return;
  }

  try {
    const normalizedRiotId = await invoke("parse_riot_id", { input: rawRiotId });

    try {
      const account = await invoke("resolve_riot_account", {
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
        title: normalizedRiotId,
        kicker: friendlySearchError(error),
        pill: "Config",
        status: "Needs key",
        region,
        level: "--",
      });
      resetChampionPool();
    }
  } catch (error) {
    setLeagueClientState({
      variant: "error",
      title: rawRiotId,
      kicker: friendlySearchError(error),
      pill: "Invalid",
      status: "Rejected",
      region,
      level: "--",
    });
    resetChampionPool();
  } finally {
    setSearchPending(false);
  }
}

async function detectLeagueClient() {
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
    const status = await invoke("league_client_status");

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

    if (status.detected) {
      setLeagueClientState({
        variant: "warning",
        title: "Client detecte",
        kicker: "Connexion locale incomplete",
        pill: "Detected",
        status: "Partial",
        region: "EUW1",
        level: "--",
      });
      resetChampionPool();
      return;
    }

    setLeagueClientState({
      variant: "offline",
      title: "GameName#TAG",
      kicker: "Aucun client League",
      pill: "Offline",
      status: "Offline",
      region: "EUW1",
      level: "--",
    });
    resetProfileIcon();
    resetChampionPool();
  } catch (error) {
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

function setLeagueClientState({ variant, title, kicker, pill, status, region, level }) {
  playerTitle.textContent = title;
  lcuKicker.textContent = kicker;
  lcuPill.textContent = pill;
  lcuPill.dataset.state = variant ?? "offline";
  lcuStatus.textContent = status;
  lcuRegion.textContent = region;
  lcuLevel.textContent = level;
}

function setSearchPending(isPending) {
  searchInput.disabled = isPending;
  searchRegion.disabled = isPending;
  searchButton.disabled = isPending;
  searchButton.textContent = isPending ? "..." : "Search";
}

function friendlySearchError(error) {
  const message = String(error ?? "Erreur recherche");

  if (message.toLowerCase().includes("riot api key")) {
    return "Cle Riot API manquante";
  }

  if (message.toLowerCase().includes("separator")) {
    return "Format attendu GameName#TAG";
  }

  return message;
}

async function hydrateConnectedProfileFromRiotId(invoke, riotId) {
  if (!riotId.includes("#")) {
    return;
  }

  try {
    const account = await invoke("resolve_riot_account", {
      input: riotId,
      platform: "EUW1",
    });

    if (account.profileIconId) {
      setProfileIcon(account.profileIconId);
    }

    if (account.summonerLevel) {
      lcuLevel.textContent = account.summonerLevel;
    }

    setChampionPool(account.championMasteries);
  } catch {
    // The LCU profile remains useful even if Riot API enrichment fails.
  }
}

async function setProfileIcon(profileIconId) {
  if (!profileIconId) {
    resetProfileIcon();
    return;
  }

  try {
    const response = await fetch("https://ddragon.leagueoflegends.com/api/versions.json");
    const versions = await response.json();
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

function resetProfileIcon() {
  avatarImg.removeAttribute("src");
  avatarImg.hidden = true;
  avatarFallback.hidden = false;
}

async function setChampionPool(masteries) {
  if (!Array.isArray(masteries) || masteries.length === 0) {
    resetChampionPool();
    return;
  }

  try {
    const championIndex = await loadChampionIndex();

    championPool.replaceChildren(
      ...masteries.slice(0, 5).map((mastery) => {
        const champion = championIndex.get(String(mastery.championId));
        const row = document.createElement("div");
        const icon = document.createElement("img");
        const body = document.createElement("div");
        const name = document.createElement("strong");
        const meta = document.createElement("span");
        const points = new Intl.NumberFormat("fr-FR").format(mastery.championPoints ?? 0);

        row.className = "champion-row";
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

function resetChampionPool() {
  championPool.replaceChildren();
  championPool.hidden = true;
  championEmpty.hidden = false;
}

async function loadChampionIndex() {
  const response = await fetch("https://ddragon.leagueoflegends.com/api/versions.json");
  const versions = await response.json();
  const version = versions[0];
  const championResponse = await fetch(
    `https://ddragon.leagueoflegends.com/cdn/${version}/data/fr_FR/champion.json`,
  );
  const championData = await championResponse.json();

  return new Map(
    Object.values(championData.data).map((champion) => [
      champion.key,
      {
        name: champion.name,
        title: champion.title,
        iconUrl: `https://ddragon.leagueoflegends.com/cdn/${version}/img/champion/${champion.image.full}`,
      },
    ]),
  );
}

async function setActiveView(viewName) {
  navItems.forEach((item) => {
    const isActive = item.dataset.navView === viewName;
    item.classList.toggle("active", isActive);
    item.toggleAttribute("aria-current", isActive);
  });

  viewPanels.forEach((panel) => {
    panel.hidden = panel.dataset.viewPanel !== viewName;
  });

  profileToolbar.hidden = viewName !== "profile";
  viewTitle.textContent = viewName === "champions" ? "Champions" : "Profil joueur";

  if (viewName === "champions") {
    await loadChampionCatalog();
  }
}

async function loadChampionCatalog() {
  if (championCatalogItems.length > 0) {
    renderChampionCatalog(championSearch.value);
    return;
  }

  const championIndex = await loadChampionIndex();
  championCatalogItems = [...championIndex.values()].sort((first, second) =>
    first.name.localeCompare(second.name, "fr"),
  );
  renderChampionCatalog(championSearch.value);
}

function renderChampionCatalog(query = "") {
  const normalizedQuery = query.trim().toLocaleLowerCase("fr");
  const champions = championCatalogItems.filter((champion) =>
    champion.name.toLocaleLowerCase("fr").includes(normalizedQuery),
  );

  championCatalog.replaceChildren(
    ...champions.map((champion) => {
      const item = document.createElement("article");
      const icon = document.createElement("img");
      const title = document.createElement("strong");
      const subtitle = document.createElement("span");

      item.className = "champion-tile";
      icon.alt = "";
      icon.src = champion.iconUrl;
      title.textContent = champion.name;
      subtitle.textContent = champion.title;
      item.append(icon, title, subtitle);
      return item;
    }),
  );
}
