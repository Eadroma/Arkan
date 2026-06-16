const shell = document.querySelector(".shell");
const toggle = document.querySelector(".sidebar-toggle");
const playerTitle = document.querySelector("[data-player-title]");
const lcuKicker = document.querySelector("[data-lcu-kicker]");
const clientDot = document.querySelector("[data-client-dot]");
const clientLabel = document.querySelector("[data-client-label]");
const clientDetail = document.querySelector("[data-client-detail]");
const lcuPill = document.querySelector("[data-lcu-pill]");
const lcuStatus = document.querySelector("[data-lcu-status]");
const lcuPort = document.querySelector("[data-lcu-port]");
const lcuLevel = document.querySelector("[data-lcu-level]");

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

detectLeagueClient();

async function detectLeagueClient() {
  const invoke = window.__TAURI__?.core?.invoke;

  if (!invoke) {
    setLeagueClientState({
      variant: "offline",
      title: "GameName#TAG",
      kicker: "Mode preview",
      label: "Tauri indisponible",
      detail: "La detection automatique fonctionne dans l'application desktop.",
      pill: "Preview",
      status: "Preview",
      port: "--",
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

      setLeagueClientState({
        variant: "online",
        title: riotId,
        kicker: "Joueur connecte detecte",
        label: "Client League connecte",
        detail: summoner.puuid ? "Profil local pret a synchroniser." : "Summoner local detecte.",
        pill: "Connected",
        status: "Connected",
        port: status.port ?? "--",
        level: summoner.summonerLevel ?? "--",
      });
      return;
    }

    if (status.detected) {
      setLeagueClientState({
        variant: "warning",
        title: "Client detecte",
        kicker: "Connexion locale incomplete",
        label: "Client trouve",
        detail: status.error ?? "Impossible de lire le joueur courant pour le moment.",
        pill: "Detected",
        status: "Partial",
        port: status.port ?? "--",
        level: "--",
      });
      return;
    }

    setLeagueClientState({
      variant: "offline",
      title: "GameName#TAG",
      kicker: "Aucun client League",
      label: "Client non detecte",
      detail: "Lance League of Legends pour reconnaitre le joueur automatiquement.",
      pill: "Offline",
      status: "Offline",
      port: "--",
      level: "--",
    });
  } catch (error) {
    setLeagueClientState({
      variant: "warning",
      title: "Detection indisponible",
      kicker: "Erreur locale",
      label: "Verification echouee",
      detail: String(error),
      pill: "Error",
      status: "Error",
      port: "--",
      level: "--",
    });
  }
}

function setLeagueClientState({ variant, title, kicker, label, detail, pill, status, port, level }) {
  playerTitle.textContent = title;
  lcuKicker.textContent = kicker;
  clientLabel.textContent = label;
  clientDetail.textContent = detail;
  lcuPill.textContent = pill;
  lcuStatus.textContent = status;
  lcuPort.textContent = port;
  lcuLevel.textContent = level;

  clientDot.classList.remove("pending", "online", "offline", "warning");
  clientDot.classList.add(variant);
}
