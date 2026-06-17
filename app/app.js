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

detectLeagueClient();

async function handleConnectedPlayerReset(event) {
  event.preventDefault();

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
