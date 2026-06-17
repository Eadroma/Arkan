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
const championRole = document.querySelector("[data-champion-role]");
const championCatalog = document.querySelector("[data-champion-catalog]");
const championBuildHero = document.querySelector(".build-hero");
const championBack = document.querySelector("[data-champion-back]");
const championDetailIcon = document.querySelector("[data-champion-detail-icon]");
const championDetailRole = document.querySelector("[data-champion-detail-role]");
const championDetailName = document.querySelector("[data-champion-detail-name]");
const championDetailTitle = document.querySelector("[data-champion-detail-title]");
const championDetailRoleSelect = document.querySelector("[data-champion-detail-role-select]");
const championAbilities = document.querySelector("[data-champion-abilities]");
const championWinrate = document.querySelector("[data-champion-winrate]");
const championPickrate = document.querySelector("[data-champion-pickrate]");
const championSample = document.querySelector("[data-champion-sample]");
const championSkillPriority = document.querySelector("[data-champion-skill-priority]");
const championSkillOrder = document.querySelector("[data-champion-skill-order]");
const championBuild = document.querySelector("[data-champion-build]");
let championCatalogItems = [];
let selectedChampion = null;

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
        id: champion.id,
        key: champion.key,
        name: champion.name,
        title: champion.title,
        tags: champion.tags,
        passive: champion.passive,
        spells: champion.spells ?? [],
        recommended: champion.recommended ?? [],
        iconUrl: `https://ddragon.leagueoflegends.com/cdn/${version}/img/champion/${champion.image.full}`,
        version,
      },
    ]),
  );
}

async function setActiveView(viewName) {
  navItems.forEach((item) => {
    const isActive =
      item.dataset.navView === viewName ||
      (viewName === "champion-detail" && item.dataset.navView === "champions");
    item.classList.toggle("active", isActive);
    item.toggleAttribute("aria-current", isActive);
  });

  viewPanels.forEach((panel) => {
    panel.hidden = panel.dataset.viewPanel !== viewName;
  });

  profileToolbar.hidden = viewName !== "profile";
  viewTitle.textContent =
    viewName === "champions" || viewName === "champion-detail" ? "Champions" : "Profil joueur";

  if (viewName === "champions" || viewName === "champion-detail") {
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
  const selectedRole = championRole.value;
  const champions = championCatalogItems.filter((champion) =>
    champion.name.toLocaleLowerCase("fr").includes(normalizedQuery) &&
    (selectedRole === "all" || champion.tags.includes(selectedRole)),
  );

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

async function openChampionDetail(championId) {
  await loadChampionCatalog();
  const champion = await loadChampionDetail(championId);
  selectedChampion = champion;
  renderChampionDetail(champion, champion.tags[0] ?? "ALL");
  await setActiveView("champion-detail");
}

async function loadChampionDetail(championId) {
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
  const details = await response.json();
  const detailedChampion = details.data[champion.id];
  const hydratedChampion = {
    ...champion,
    passive: detailedChampion.passive,
    spells: detailedChampion.spells ?? [],
    recommended: detailedChampion.recommended ?? [],
  };

  championCatalogItems = championCatalogItems.map((item) =>
    item.id === champion.id ? hydratedChampion : item,
  );

  return hydratedChampion;
}

function renderChampionDetail(champion, role) {
  championDetailIcon.src = champion.iconUrl;
  championDetailName.textContent = champion.name;
  championDetailTitle.textContent = champion.title;
  championDetailRole.textContent = roleLabel(role);
  championBuildHero.style.setProperty(
    "--champion-splash",
    `url("https://ddragon.leagueoflegends.com/cdn/img/champion/splash/${champion.id}_0.jpg")`,
  );
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

  renderAbilityStrip(champion);
  renderSkillOrder(champion);
  renderBuild(champion);
}

function renderAbilityStrip(champion) {
  const abilities = [
    { key: "P", name: champion.passive?.name ?? "Passive", image: champion.passive?.image?.full, passive: true },
    ...champion.spells.map((spell, index) => ({
      key: ["Q", "W", "E", "R"][index] ?? "?",
      name: spell.name,
      image: spell.image?.full,
      passive: false,
    })),
  ];

  championAbilities.replaceChildren(
    ...abilities.map((ability) => {
      const item = document.createElement("div");
      const img = document.createElement("img");
      const key = document.createElement("span");
      const imagePath = ability.passive ? "passive" : "spell";

      item.className = "ability-chip";
      img.alt = ability.name;
      img.src = ability.image
        ? `https://ddragon.leagueoflegends.com/cdn/${champion.version}/img/${imagePath}/${ability.image}`
        : champion.iconUrl;
      key.textContent = ability.key;
      item.append(img, key);
      return item;
    }),
  );
}

function renderSkillOrder(champion) {
  const primarySpells = champion.spells.slice(0, 3);
  const skillPath = ["Q", "E", "W", "Q", "Q", "R", "Q", "W", "Q", "W", "R", "W", "W", "E", "E", "R", "E", "E"];
  const skillIcons = primarySpells.map((spell, index) => ({ spell, key: ["Q", "W", "E"][index] ?? "?" }));

  championSkillPriority.replaceChildren(
    ...skillIcons.map(({ spell, key }) => {
      const item = document.createElement("div");
      const img = document.createElement("img");
      const label = document.createElement("span");

      item.className = "priority-icon";
      img.alt = spell.name;
      img.src = spell.image?.full
        ? `https://ddragon.leagueoflegends.com/cdn/${champion.version}/img/spell/${spell.image.full}`
        : champion.iconUrl;
      label.textContent = key;
      item.append(img, label);
      return item;
    }),
  );

  championSkillOrder.replaceChildren(
    ...skillIcons.map(({ spell, key: spellKey }) => {
      const item = document.createElement("div");
      const key = document.createElement("strong");
      const name = document.createElement("span");
      const levels = document.createElement("div");

      item.className = "skill-step";
      levels.className = "skill-levels";
      key.textContent = spellKey;
      name.textContent = spell.name;
      levels.replaceChildren(
        ...skillPath.map((level, levelIndex) => {
          const cell = document.createElement("span");
          cell.textContent = level === spellKey ? String(levelIndex + 1) : "";
          cell.dataset.active = String(level === spellKey);
          return cell;
        }),
      );
      item.append(key, name, levels);
      return item;
    }),
  );

  if (primarySpells.length === 0) {
    championSkillOrder.textContent = "A synchroniser depuis les timelines MATCH-V5.";
  }
}

function renderBuild(champion) {
  const build = extractRecommendedBuild(champion.recommended);

  championBuild.replaceChildren(
    ...["Starting", "Core", "Options"].map((label, sectionIndex) => {
      const section = document.createElement("div");
      const heading = document.createElement("span");
      const items = document.createElement("div");
      const slice = build.slice(sectionIndex * 2, sectionIndex * 2 + 2);

      section.className = "build-section";
      items.className = "build-items";
      heading.textContent = label;
      items.replaceChildren(
        ...(slice.length > 0 ? slice : ["1001", "1056"]).map((itemId) => {
          const item = document.createElement("img");
          item.alt = `Item ${itemId}`;
          item.src = `https://ddragon.leagueoflegends.com/cdn/${champion.version}/img/item/${itemId}.png`;
          return item;
        }),
      );
      section.append(heading, items);
      return section;
    }),
  );

  if (build.length === 0) {
    championBuild.textContent = "A synchroniser depuis les matchs MATCH-V5.";
  }
}

function extractRecommendedBuild(recommended) {
  const firstBlock = recommended
    .flatMap((recommendation) => recommendation.blocks ?? [])
    .find((block) => Array.isArray(block.items) && block.items.length > 0);

  return (firstBlock?.items ?? []).slice(0, 6).map((item) => item.id);
}

function roleLabel(role) {
  const labels = {
    Assassin: "Assassin",
    Fighter: "Combattant",
    Mage: "Mage",
    Marksman: "Tireur",
    Support: "Support",
    Tank: "Tank",
    ALL: "Tous roles",
  };

  return labels[role] ?? role;
}
