use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::Value;

const CHAMPION_SAMPLE_MATCH_LIMIT: u16 = 500;
const MATCH_V5_PAGE_SIZE: u8 = 100;

#[tauri::command]
fn parse_riot_id(input: &str) -> Result<String, String> {
    arkan_core::RiotId::parse(input)
        .map(|riot_id| riot_id.to_string())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn config_status() -> Result<String, String> {
    let config = arkan_core::AppConfig::from_env().map_err(|error| error.to_string())?;
    let api_key_state = config
        .masked_riot_api_key()
        .unwrap_or_else(|| "missing".to_owned());

    Ok(format!(
        "api_key={api_key_state}; platform={}; language={}",
        config.default_platform(),
        config.default_language()
    ))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RiotAccountResponse {
    puuid: String,
    game_name: String,
    tag_line: String,
    profile_icon_id: Option<u32>,
    summoner_level: Option<u32>,
    champion_masteries: Vec<RiotChampionMasteryResponse>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RiotChampionMasteryResponse {
    champion_id: u32,
    champion_level: u32,
    champion_points: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChampionRoleStatsResponse {
    champion_id: u32,
    champion_key: String,
    champion_name: String,
    patch: String,
    pick_rate: f64,
    platform_id: String,
    queue_id: u32,
    role: String,
    sample_size: u32,
    source: String,
    tier: Option<String>,
    win_rate: f64,
    wins: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChampionSpellPairStatsResponse {
    champion_id: u32,
    games: u32,
    source: String,
    spell_ids: [u32; 2],
    win_rate: f64,
    wins: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChampionRunePageStatsResponse {
    champion_id: u32,
    games: u32,
    primary_style_id: u32,
    selected_perk_ids: Vec<u32>,
    source: String,
    sub_style_id: u32,
    win_rate: f64,
    wins: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChampionSampleSyncResponse {
    fetched_matches: usize,
    requested_matches: u16,
    source: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MatchHistoryEntry {
    assists: u32,
    champion_id: u32,
    champion_name: String,
    deaths: u32,
    duration_seconds: u64,
    game_created_at: i64,
    kills: u32,
    lp_delta: Option<i32>,
    match_id: String,
    queue_id: u32,
    role: String,
    win: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MatchDetailResponse {
    duration_seconds: u64,
    game_created_at: i64,
    match_id: String,
    queue_id: u32,
    teams: Vec<MatchTeam>,
    timeline: Vec<MatchTimelinePoint>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MatchTeam {
    participants: Vec<MatchParticipant>,
    result: String,
    team_id: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MatchParticipant {
    assists: u32,
    champion_id: u32,
    champion_level: u32,
    champion_name: String,
    cs: u32,
    deaths: u32,
    gold_earned: u32,
    items: Vec<u32>,
    kills: u32,
    participant_id: u32,
    riot_id: String,
    summoner_spell_ids: Vec<u32>,
    team_position: String,
    total_damage_to_champions: u32,
    vision_score: u32,
    win: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MatchTimelinePoint {
    blue_damage: u32,
    blue_gold: u32,
    blue_xp: u32,
    minute: u32,
    red_damage: u32,
    red_gold: u32,
    red_xp: u32,
}

#[tauri::command]
async fn resolve_riot_account(input: &str, platform: &str) -> Result<RiotAccountResponse, String> {
    let config = arkan_core::AppConfig::from_env().map_err(|error| error.to_string())?;
    let api_key = config
        .riot_api_key()
        .ok_or_else(|| "Riot API key is missing".to_owned())?;
    let riot_id = arkan_core::RiotId::parse(input).map_err(|error| error.to_string())?;
    let platform = platform
        .parse::<arkan_core::PlatformRoute>()
        .map_err(|error| error.to_string())?;
    let client = arkan_core::RiotApiClient::new(api_key).map_err(|error| error.to_string())?;
    let account = client
        .account_by_riot_id(platform.regional_route(), &riot_id)
        .await
        .map_err(|error| error.to_string())?;
    let summoner = client
        .summoner_by_puuid(platform, &account.puuid)
        .await
        .ok();
    let champion_masteries = client
        .champion_mastery_top(platform, &account.puuid, 5)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|mastery| RiotChampionMasteryResponse {
            champion_id: mastery.champion_id,
            champion_level: mastery.champion_level,
            champion_points: mastery.champion_points,
        })
        .collect();

    Ok(RiotAccountResponse {
        puuid: account.puuid,
        game_name: account.game_name,
        tag_line: account.tag_line,
        profile_icon_id: summoner.as_ref().map(|summoner| summoner.profile_icon_id),
        summoner_level: summoner.as_ref().map(|summoner| summoner.summoner_level),
        champion_masteries,
    })
}

#[tauri::command]
async fn match_detail(match_id: &str, platform: &str) -> Result<MatchDetailResponse, String> {
    let config = arkan_core::AppConfig::from_env().map_err(|error| error.to_string())?;
    let api_key = config
        .riot_api_key()
        .ok_or_else(|| "Riot API key is missing".to_owned())?;
    let platform = platform
        .parse::<arkan_core::PlatformRoute>()
        .map_err(|error| error.to_string())?;
    let client = arkan_core::RiotApiClient::new(api_key).map_err(|error| error.to_string())?;
    let route = platform.regional_route();
    let detail = client
        .match_by_id(route, match_id)
        .await
        .map_err(|error| error.to_string())?;
    let timeline = client
        .match_timeline_by_id(route, match_id)
        .await
        .map_err(|error| error.to_string())?;

    match_detail_response_from_values(&detail, &timeline)
        .ok_or_else(|| "Unable to parse match detail".to_owned())
}

#[tauri::command]
async fn match_history(
    input: &str,
    platform: &str,
    start: Option<u32>,
    count: Option<u8>,
) -> Result<Vec<MatchHistoryEntry>, String> {
    let config = arkan_core::AppConfig::from_env().map_err(|error| error.to_string())?;
    let api_key = config
        .riot_api_key()
        .ok_or_else(|| "Riot API key is missing".to_owned())?;
    let riot_id = arkan_core::RiotId::parse(input).map_err(|error| error.to_string())?;
    let platform = platform
        .parse::<arkan_core::PlatformRoute>()
        .map_err(|error| error.to_string())?;
    let client = arkan_core::RiotApiClient::new(api_key).map_err(|error| error.to_string())?;
    let start = start.unwrap_or(0);
    let count = count.unwrap_or(10).clamp(1, MATCH_V5_PAGE_SIZE);
    let account = client
        .account_by_riot_id(platform.regional_route(), &riot_id)
        .await
        .map_err(|error| error.to_string())?;
    let match_ids = client
        .match_ids_by_puuid(platform.regional_route(), &account.puuid, start, count)
        .await
        .map_err(|error| error.to_string())?;
    let mut fetched_matches = Vec::new();

    for match_id in match_ids {
        let detail = client
            .match_by_id(platform.regional_route(), &match_id)
            .await
            .map_err(|error| error.to_string())?;

        if let Some(entry) = match_history_entry_from_detail(&detail, &account.puuid) {
            fetched_matches.push((detail, entry));
        }
    }

    persist_match_history(&account, platform, &fetched_matches)?;

    Ok(fetched_matches
        .into_iter()
        .map(|(_, entry)| entry)
        .collect())
}

#[tauri::command]
async fn sync_champion_sample(
    input: &str,
    platform: &str,
    requested_matches: Option<u16>,
) -> Result<ChampionSampleSyncResponse, String> {
    let config = arkan_core::AppConfig::from_env().map_err(|error| error.to_string())?;
    let api_key = config
        .riot_api_key()
        .ok_or_else(|| "Riot API key is missing".to_owned())?;
    let riot_id = arkan_core::RiotId::parse(input).map_err(|error| error.to_string())?;
    let platform = platform
        .parse::<arkan_core::PlatformRoute>()
        .map_err(|error| error.to_string())?;
    let client = arkan_core::RiotApiClient::new(api_key).map_err(|error| error.to_string())?;
    let requested_matches = requested_matches
        .unwrap_or(CHAMPION_SAMPLE_MATCH_LIMIT)
        .clamp(1, CHAMPION_SAMPLE_MATCH_LIMIT);
    let account = client
        .account_by_riot_id(platform.regional_route(), &riot_id)
        .await
        .map_err(|error| error.to_string())?;
    let mut fetched_matches = Vec::new();
    let mut start = 0_u32;

    while fetched_matches.len() < usize::from(requested_matches) {
        let remaining = usize::from(requested_matches) - fetched_matches.len();
        let count = remaining.min(usize::from(MATCH_V5_PAGE_SIZE)) as u8;
        let match_ids = client
            .match_ids_by_puuid(platform.regional_route(), &account.puuid, start, count)
            .await
            .map_err(|error| error.to_string())?;

        if match_ids.is_empty() {
            break;
        }

        start += u32::try_from(match_ids.len()).unwrap_or(0);

        for match_id in match_ids {
            let detail = client
                .match_by_id(platform.regional_route(), &match_id)
                .await
                .map_err(|error| error.to_string())?;

            if let Some(entry) = match_history_entry_from_detail(&detail, &account.puuid) {
                fetched_matches.push((detail, entry));
            }

            if fetched_matches.len() >= usize::from(requested_matches) {
                break;
            }
        }
    }

    persist_match_history(&account, platform, &fetched_matches)?;

    Ok(ChampionSampleSyncResponse {
        fetched_matches: fetched_matches.len(),
        requested_matches,
        source: "sample-match-v5".to_owned(),
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LeagueClientStatus {
    detected: bool,
    connected: bool,
    cached: bool,
    database_path: Option<String>,
    lockfile_path: Option<String>,
    port: Option<u16>,
    protocol: Option<String>,
    summoner: Option<CurrentSummoner>,
    error: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CurrentSummoner {
    display_name: String,
    game_name: Option<String>,
    tag_line: Option<String>,
    puuid: Option<String>,
    summoner_id: Option<u64>,
    profile_icon_id: Option<u32>,
    summoner_level: Option<u32>,
    champion_masteries: Vec<RiotChampionMasteryResponse>,
}

impl CurrentSummoner {
    fn has_visible_identity(&self) -> bool {
        (!self.display_name.trim().is_empty() && self.display_name != "Connected summoner")
            || self
                .game_name
                .as_deref()
                .is_some_and(|game_name| !game_name.trim().is_empty())
    }
}

#[derive(Clone, Debug)]
struct LcuHydratedSummonerCacheEntry {
    hydrated_at: Instant,
    identity_key: String,
    session_key: String,
    summoner: CurrentSummoner,
}

static LCU_HYDRATED_SUMMONER_CACHE: OnceLock<Mutex<Option<LcuHydratedSummonerCacheEntry>>> =
    OnceLock::new();
static LCU_PERSISTED_SUMMONER_CACHE: OnceLock<Mutex<Option<String>>> = OnceLock::new();

const LCU_HYDRATED_SUMMONER_CACHE_TTL: Duration = Duration::from_secs(30);

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct LcuCurrentSummoner {
    display_name: Option<String>,
    game_name: Option<String>,
    tag_line: Option<String>,
    puuid: Option<String>,
    summoner_id: Option<u64>,
    profile_icon_id: Option<u32>,
    summoner_level: Option<u32>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct LcuChatMe {
    game_name: Option<String>,
    game_tag: Option<String>,
    name: Option<String>,
    pid: Option<String>,
    puuid: Option<String>,
    summoner_id: Option<LcuNumericId>,
    icon: Option<LcuNumericId>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct LcuChampionMastery {
    champion_id: u32,
    champion_level: u32,
    champion_points: u32,
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum LcuNumericId {
    Number(u64),
    Text(String),
}

impl LcuNumericId {
    fn as_u64(&self) -> Option<u64> {
        match self {
            Self::Number(value) => Some(*value),
            Self::Text(value) => value.parse::<u64>().ok(),
        }
    }
}

#[tauri::command]
async fn league_client_status() -> LeagueClientStatus {
    let lockfile_path = match find_lockfile_path() {
        Some(path) => path,
        None => {
            return LeagueClientStatus {
                detected: false,
                connected: false,
                cached: false,
                database_path: app_database_path()
                    .ok()
                    .map(|path| path.display().to_string()),
                lockfile_path: None,
                port: None,
                protocol: None,
                summoner: None,
                error: Some("League Client lockfile not found".to_owned()),
            };
        }
    };

    let lockfile_content = match fs::read_to_string(&lockfile_path) {
        Ok(content) => content,
        Err(error) => {
            return LeagueClientStatus {
                detected: false,
                connected: false,
                cached: false,
                database_path: app_database_path()
                    .ok()
                    .map(|path| path.display().to_string()),
                lockfile_path: Some(lockfile_path.display().to_string()),
                port: None,
                protocol: None,
                summoner: None,
                error: Some(format!("failed to read League Client lockfile: {error}")),
            };
        }
    };

    let lockfile = match arkan_core::LeagueClientLockfile::parse(&lockfile_content) {
        Ok(lockfile) => lockfile,
        Err(error) => {
            return LeagueClientStatus {
                detected: true,
                connected: false,
                cached: false,
                database_path: app_database_path()
                    .ok()
                    .map(|path| path.display().to_string()),
                lockfile_path: Some(lockfile_path.display().to_string()),
                port: None,
                protocol: None,
                summoner: None,
                error: Some(error.to_string()),
            };
        }
    };

    let summoner_result = fetch_current_summoner(&lockfile).await;

    match summoner_result {
        Ok(mut summoner) => {
            hydrate_current_summoner_champion_pool_with_cache(
                &lockfile_path,
                &lockfile,
                &mut summoner,
            )
            .await;
            let cache_result = persist_current_summoner(&summoner);
            let database_path = app_database_path()
                .ok()
                .map(|path| path.display().to_string());

            LeagueClientStatus {
                detected: true,
                connected: true,
                cached: cache_result.is_ok(),
                database_path,
                lockfile_path: Some(lockfile_path.display().to_string()),
                port: Some(lockfile.port()),
                protocol: Some(lockfile.protocol().to_string()),
                summoner: Some(summoner),
                error: cache_result.err(),
            }
        }
        Err(error) => LeagueClientStatus {
            detected: true,
            connected: false,
            cached: false,
            database_path: app_database_path()
                .ok()
                .map(|path| path.display().to_string()),
            lockfile_path: Some(lockfile_path.display().to_string()),
            port: Some(lockfile.port()),
            protocol: Some(lockfile.protocol().to_string()),
            summoner: None,
            error: Some(error),
        },
    }
}

#[tauri::command]
fn local_database_status() -> Result<String, String> {
    let path = app_database_path()?;
    let connection = open_app_database()?;
    let version = arkan_core::schema_version(&connection).map_err(|error| error.to_string())?;

    Ok(format!("path={}; schema_version={version}", path.display()))
}

#[tauri::command]
fn refresh_champion_role_stats(platform: &str, tier: Option<&str>) -> Result<usize, String> {
    let connection = open_app_database()?;
    let platform = platform
        .parse::<arkan_core::PlatformRoute>()
        .map_err(|error| error.to_string())?;
    let stats =
        arkan_core::refresh_local_champion_role_stats(&connection, &platform.to_string(), tier)
            .map_err(|error| error.to_string())?;

    Ok(stats.len())
}

#[tauri::command]
fn champion_role_stats(
    champion_id: u32,
    platform: &str,
) -> Result<Vec<ChampionRoleStatsResponse>, String> {
    let connection = open_app_database()?;
    let platform = platform
        .parse::<arkan_core::PlatformRoute>()
        .map_err(|error| error.to_string())?;
    let stats = arkan_core::find_champion_role_stats_by_champion(
        &connection,
        champion_id,
        &platform.to_string(),
    )
    .map_err(|error| error.to_string())?;

    Ok(stats
        .into_iter()
        .map(ChampionRoleStatsResponse::from)
        .collect())
}

#[tauri::command]
fn champion_spell_pairs(champion_id: u32) -> Result<Vec<ChampionSpellPairStatsResponse>, String> {
    let connection = open_app_database()?;
    let pairs = arkan_core::find_local_champion_spell_pairs(&connection, champion_id)
        .map_err(|error| error.to_string())?;

    Ok(pairs
        .into_iter()
        .map(ChampionSpellPairStatsResponse::from)
        .collect())
}

#[tauri::command]
fn champion_rune_pages(champion_id: u32) -> Result<Vec<ChampionRunePageStatsResponse>, String> {
    let connection = open_app_database()?;
    let pages = arkan_core::find_local_champion_rune_pages(&connection, champion_id)
        .map_err(|error| error.to_string())?;

    Ok(pages
        .into_iter()
        .map(ChampionRunePageStatsResponse::from)
        .collect())
}

impl From<arkan_core::ChampionRoleStats> for ChampionRoleStatsResponse {
    fn from(stats: arkan_core::ChampionRoleStats) -> Self {
        Self {
            champion_id: stats.champion_id,
            champion_key: stats.champion_key,
            champion_name: stats.champion_name,
            patch: stats.patch,
            pick_rate: stats.pick_rate,
            platform_id: stats.platform_id,
            queue_id: stats.queue_id,
            role: stats.role,
            sample_size: stats.sample_size,
            source: stats.source,
            tier: stats.tier,
            win_rate: stats.win_rate,
            wins: stats.wins,
        }
    }
}

impl From<arkan_core::ChampionSpellPairStats> for ChampionSpellPairStatsResponse {
    fn from(stats: arkan_core::ChampionSpellPairStats) -> Self {
        Self {
            champion_id: stats.champion_id,
            games: stats.games,
            source: stats.source,
            spell_ids: stats.spell_ids,
            win_rate: stats.win_rate,
            wins: stats.wins,
        }
    }
}

impl From<arkan_core::ChampionRunePageStats> for ChampionRunePageStatsResponse {
    fn from(stats: arkan_core::ChampionRunePageStats) -> Self {
        Self {
            champion_id: stats.champion_id,
            games: stats.games,
            primary_style_id: stats.primary_style_id,
            selected_perk_ids: stats.selected_perk_ids,
            source: stats.source,
            sub_style_id: stats.sub_style_id,
            win_rate: stats.win_rate,
            wins: stats.wins,
        }
    }
}

fn find_lockfile_path() -> Option<PathBuf> {
    let configured_path = env::var("ARKAN_LCU_LOCKFILE_PATH")
        .ok()
        .map(PathBuf::from)
        .filter(|path| path.exists());

    configured_path.or_else(|| {
        let default_path = PathBuf::from(r"C:\Riot Games\League of Legends\lockfile");
        default_path.exists().then_some(default_path)
    })
}

fn app_database_path() -> Result<PathBuf, String> {
    if let Ok(path) = env::var("ARKAN_DB_PATH") {
        return Ok(PathBuf::from(path));
    }

    let app_data = env::var("APPDATA")
        .map(PathBuf::from)
        .map_err(|error| format!("APPDATA is unavailable: {error}"))?;

    Ok(app_data.join("Arkan").join("arkan.sqlite3"))
}

fn open_app_database() -> Result<rusqlite::Connection, String> {
    let path = app_database_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create database directory: {error}"))?;
    }

    open_database_at(&path)
}

fn open_database_at(path: &Path) -> Result<rusqlite::Connection, String> {
    let mut connection =
        rusqlite::Connection::open(path).map_err(|error| format!("failed to open DB: {error}"))?;
    arkan_core::migrate(&mut connection).map_err(|error| error.to_string())?;
    Ok(connection)
}

fn persist_current_summoner(summoner: &CurrentSummoner) -> Result<(), String> {
    let path = app_database_path()?;

    persist_current_summoner_at(&path, summoner)
}

fn persist_current_summoner_at(path: &Path, summoner: &CurrentSummoner) -> Result<(), String> {
    let persist_key = current_summoner_persist_key(summoner)?;

    if recently_persisted_current_summoner_key(&persist_key) {
        return Ok(());
    }

    let puuid = summoner
        .puuid
        .clone()
        .ok_or_else(|| "current summoner has no PUUID; cache skipped".to_owned())?;
    let game_name = summoner
        .game_name
        .clone()
        .unwrap_or_else(|| summoner.display_name.clone());
    let tag_line = summoner
        .tag_line
        .clone()
        .unwrap_or_else(|| "LOCAL".to_owned());
    let player = arkan_core::PlayerRecord {
        puuid,
        game_name,
        tag_line,
        platform_id: "EUW1".to_owned(),
        summoner_id: summoner.summoner_id.map(|id| id.to_string()),
        account_id: None,
        summoner_level: summoner.summoner_level,
        profile_icon_id: summoner.profile_icon_id,
    };
    let connection = open_database_at(path)?;

    arkan_core::upsert_player(&connection, &player).map_err(|error| error.to_string())?;
    store_persisted_current_summoner_key(&persist_key);
    Ok(())
}

fn persist_match_history(
    account: &arkan_core::RiotAccount,
    platform: arkan_core::PlatformRoute,
    fetched_matches: &[(Value, MatchHistoryEntry)],
) -> Result<(), String> {
    let connection = open_app_database()?;
    let player = arkan_core::PlayerRecord {
        puuid: account.puuid.clone(),
        game_name: account.game_name.clone(),
        tag_line: account.tag_line.clone(),
        platform_id: platform.to_string(),
        summoner_id: None,
        account_id: None,
        summoner_level: None,
        profile_icon_id: None,
    };

    arkan_core::upsert_player(&connection, &player).map_err(|error| error.to_string())?;

    for (detail, entry) in fetched_matches {
        let match_record = match_record_from_detail(detail, platform.regional_route())?;
        let player_match = player_match_record_from_detail(detail, &account.puuid, entry)?;

        arkan_core::upsert_match(&connection, &match_record).map_err(|error| error.to_string())?;
        arkan_core::upsert_player_match(&connection, &player_match)
            .map_err(|error| error.to_string())?;
    }

    arkan_core::refresh_local_champion_role_stats(&connection, &platform.to_string(), None)
        .map_err(|error| error.to_string())?;

    Ok(())
}

fn match_record_from_detail(
    detail: &Value,
    route: arkan_core::RegionalRoute,
) -> Result<arkan_core::MatchRecord, String> {
    let metadata = detail
        .get("metadata")
        .ok_or_else(|| "match metadata is missing".to_owned())?;
    let info = detail
        .get("info")
        .ok_or_else(|| "match info is missing".to_owned())?;
    let match_id = metadata
        .get("matchId")
        .and_then(Value::as_str)
        .ok_or_else(|| "match id is missing".to_owned())?;

    Ok(arkan_core::MatchRecord {
        match_id: match_id.to_owned(),
        regional_route: route.to_string(),
        game_creation: info.get("gameCreation").and_then(Value::as_i64),
        game_duration: info.get("gameDuration").and_then(Value::as_u64),
        queue_id: info
            .get("queueId")
            .and_then(Value::as_u64)
            .and_then(|value| value.try_into().ok()),
        game_version: info
            .get("gameVersion")
            .and_then(Value::as_str)
            .map(str::to_owned),
        raw_json: serde_json::to_string(detail).map_err(|error| error.to_string())?,
    })
}

fn player_match_record_from_detail(
    detail: &Value,
    puuid: &str,
    entry: &MatchHistoryEntry,
) -> Result<arkan_core::PlayerMatchRecord, String> {
    let participant = detail
        .get("info")
        .and_then(|info| info.get("participants"))
        .and_then(Value::as_array)
        .and_then(|participants| {
            participants
                .iter()
                .find(|participant| participant.get("puuid").and_then(Value::as_str) == Some(puuid))
        })
        .ok_or_else(|| "player participant is missing".to_owned())?;

    Ok(arkan_core::PlayerMatchRecord {
        puuid: puuid.to_owned(),
        match_id: entry.match_id.clone(),
        champion_id: entry.champion_id,
        champion_name: Some(entry.champion_name.clone()),
        team_position: Some(entry.role.clone()),
        win: entry.win,
        kills: entry.kills,
        deaths: entry.deaths,
        assists: entry.assists,
        total_cs: value_u32(participant, "totalMinionsKilled")
            + value_u32(participant, "neutralMinionsKilled"),
        gold_earned: value_u32(participant, "goldEarned"),
        vision_score: value_u32(participant, "visionScore"),
    })
}

async fn fetch_current_summoner(
    lockfile: &arkan_core::LeagueClientLockfile,
) -> Result<CurrentSummoner, String> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|error| format!("failed to create LCU HTTP client: {error}"))?;
    let current_result = fetch_current_summoner_endpoint(&client, lockfile).await;

    match current_result {
        Ok(summoner) if summoner.has_visible_identity() => Ok(summoner),
        Ok(summoner) => fetch_chat_profile_endpoint(&client, lockfile)
            .await
            .or(Ok(summoner)),
        Err(current_error) => fetch_chat_profile_endpoint(&client, lockfile)
            .await
            .map_err(|chat_error| format!("{current_error}; fallback failed: {chat_error}")),
    }
}

async fn fetch_current_summoner_endpoint(
    client: &reqwest::Client,
    lockfile: &arkan_core::LeagueClientLockfile,
) -> Result<CurrentSummoner, String> {
    let url = format!("{}/lol-summoner/v1/current-summoner", lockfile.base_url());

    let response = client
        .get(url)
        .basic_auth("riot", Some(lockfile.password()))
        .send()
        .await
        .map_err(|error| format!("failed to call League Client API: {error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "League Client API returned HTTP {}",
            response.status()
        ));
    }

    let summoner = response
        .json::<LcuCurrentSummoner>()
        .await
        .map_err(|error| format!("failed to parse current summoner response: {error}"))?;

    Ok(CurrentSummoner {
        display_name: summoner
            .display_name
            .or_else(|| summoner.game_name.clone())
            .unwrap_or_else(|| "Connected summoner".to_owned()),
        game_name: summoner.game_name,
        tag_line: summoner.tag_line,
        puuid: summoner.puuid,
        summoner_id: summoner.summoner_id,
        profile_icon_id: summoner.profile_icon_id,
        summoner_level: summoner.summoner_level,
        champion_masteries: Vec::new(),
    })
}

async fn fetch_chat_profile_endpoint(
    client: &reqwest::Client,
    lockfile: &arkan_core::LeagueClientLockfile,
) -> Result<CurrentSummoner, String> {
    let url = format!("{}/lol-chat/v1/me", lockfile.base_url());
    let response = client
        .get(url)
        .basic_auth("riot", Some(lockfile.password()))
        .send()
        .await
        .map_err(|error| format!("failed to call League Client chat API: {error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "League Client chat API returned HTTP {}",
            response.status()
        ));
    }

    let chat_profile = response
        .json::<LcuChatMe>()
        .await
        .map_err(|error| format!("failed to parse League Client chat profile: {error}"))?;

    Ok(current_summoner_from_chat_profile(chat_profile))
}

fn current_summoner_from_chat_profile(chat_profile: LcuChatMe) -> CurrentSummoner {
    let display_name = chat_profile
        .name
        .clone()
        .or_else(|| {
            chat_profile
                .game_name
                .as_ref()
                .zip(chat_profile.game_tag.as_ref())
                .map(|(game_name, game_tag)| format!("{game_name}#{game_tag}"))
        })
        .or(chat_profile.pid.clone())
        .or(chat_profile.game_name.clone())
        .unwrap_or_else(|| "Connected summoner".to_owned());

    CurrentSummoner {
        display_name,
        game_name: chat_profile.game_name,
        tag_line: chat_profile.game_tag,
        puuid: chat_profile.puuid,
        summoner_id: chat_profile
            .summoner_id
            .as_ref()
            .and_then(LcuNumericId::as_u64),
        profile_icon_id: chat_profile
            .icon
            .as_ref()
            .and_then(LcuNumericId::as_u64)
            .and_then(|icon| u32::try_from(icon).ok()),
        summoner_level: None,
        champion_masteries: Vec::new(),
    }
}

async fn hydrate_current_summoner_champion_pool(
    lockfile: &arkan_core::LeagueClientLockfile,
    summoner: &mut CurrentSummoner,
) {
    if hydrate_current_summoner_lcu_champion_pool(lockfile, summoner)
        .await
        .is_ok()
    {
        return;
    }

    let Ok(config) = arkan_core::AppConfig::from_env() else {
        return;
    };
    let Some(api_key) = config.riot_api_key() else {
        return;
    };
    let Ok(client) = arkan_core::RiotApiClient::new(api_key) else {
        return;
    };

    hydrate_current_summoner_account_identity(&client, summoner).await;

    let Some(puuid) = summoner.puuid.as_deref() else {
        return;
    };
    let platform = config.default_platform();
    let Ok(masteries) = client.champion_mastery_top(platform, puuid, 5).await else {
        return;
    };

    summoner.champion_masteries = masteries
        .into_iter()
        .map(|mastery| RiotChampionMasteryResponse {
            champion_id: mastery.champion_id,
            champion_level: mastery.champion_level,
            champion_points: mastery.champion_points,
        })
        .collect();
}

async fn hydrate_current_summoner_champion_pool_with_cache(
    lockfile_path: &Path,
    lockfile: &arkan_core::LeagueClientLockfile,
    summoner: &mut CurrentSummoner,
) {
    let session_key = lcu_session_key(lockfile_path, lockfile);
    let Some(identity_key) = current_summoner_identity_key(summoner) else {
        hydrate_current_summoner_champion_pool(lockfile, summoner).await;
        return;
    };

    if let Some(cached_summoner) = cached_hydrated_current_summoner(&session_key, &identity_key) {
        *summoner = cached_summoner;
        return;
    }

    hydrate_current_summoner_champion_pool(lockfile, summoner).await;
    store_hydrated_current_summoner(&session_key, &identity_key, summoner);
}

fn cached_hydrated_current_summoner(
    session_key: &str,
    identity_key: &str,
) -> Option<CurrentSummoner> {
    let cache = LCU_HYDRATED_SUMMONER_CACHE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .ok()?;
    let entry = cache.as_ref()?;

    if entry.session_key == session_key
        && entry.identity_key == identity_key
        && entry.hydrated_at.elapsed() <= LCU_HYDRATED_SUMMONER_CACHE_TTL
    {
        return Some(entry.summoner.clone());
    }

    None
}

fn store_hydrated_current_summoner(
    session_key: &str,
    identity_key: &str,
    summoner: &CurrentSummoner,
) {
    let Ok(mut cache) = LCU_HYDRATED_SUMMONER_CACHE
        .get_or_init(|| Mutex::new(None))
        .lock()
    else {
        return;
    };

    *cache = Some(LcuHydratedSummonerCacheEntry {
        hydrated_at: Instant::now(),
        identity_key: identity_key.to_owned(),
        session_key: session_key.to_owned(),
        summoner: summoner.clone(),
    });
}

fn lcu_session_key(lockfile_path: &Path, lockfile: &arkan_core::LeagueClientLockfile) -> String {
    format!(
        "{}:{}:{}",
        lockfile_path.display(),
        lockfile.protocol(),
        lockfile.port()
    )
}

fn current_summoner_identity_key(summoner: &CurrentSummoner) -> Option<String> {
    if let Some(puuid) = summoner.puuid.as_deref().filter(|value| !value.is_empty()) {
        return Some(format!("puuid:{puuid}"));
    }

    if let Some(summoner_id) = summoner.summoner_id {
        return Some(format!("summoner:{summoner_id}"));
    }

    let game_name = summoner.game_name.as_deref()?.trim();
    let tag_line = summoner.tag_line.as_deref()?.trim();

    if game_name.is_empty() || tag_line.is_empty() {
        return None;
    }

    Some(format!(
        "riot-id:{}#{}",
        game_name.to_lowercase(),
        tag_line.to_lowercase()
    ))
}

fn current_summoner_persist_key(summoner: &CurrentSummoner) -> Result<String, String> {
    let puuid = summoner
        .puuid
        .as_deref()
        .ok_or_else(|| "current summoner has no PUUID; cache skipped".to_owned())?;
    let game_name = summoner
        .game_name
        .as_deref()
        .unwrap_or(&summoner.display_name);
    let tag_line = summoner.tag_line.as_deref().unwrap_or("LOCAL");
    let summoner_id = summoner
        .summoner_id
        .map(|value| value.to_string())
        .unwrap_or_default();
    let summoner_level = summoner
        .summoner_level
        .map(|value| value.to_string())
        .unwrap_or_default();
    let profile_icon_id = summoner
        .profile_icon_id
        .map(|value| value.to_string())
        .unwrap_or_default();

    Ok(format!(
        "{puuid}|{game_name}|{tag_line}|EUW1|{summoner_id}|{summoner_level}|{profile_icon_id}"
    ))
}

fn recently_persisted_current_summoner_key(persist_key: &str) -> bool {
    let Ok(cache) = LCU_PERSISTED_SUMMONER_CACHE
        .get_or_init(|| Mutex::new(None))
        .lock()
    else {
        return false;
    };

    cache.as_deref() == Some(persist_key)
}

fn store_persisted_current_summoner_key(persist_key: &str) {
    let Ok(mut cache) = LCU_PERSISTED_SUMMONER_CACHE
        .get_or_init(|| Mutex::new(None))
        .lock()
    else {
        return;
    };

    *cache = Some(persist_key.to_owned());
}

async fn hydrate_current_summoner_lcu_champion_pool(
    lockfile: &arkan_core::LeagueClientLockfile,
    summoner: &mut CurrentSummoner,
) -> Result<(), String> {
    let Some(summoner_id) = summoner.summoner_id else {
        return Err("current summoner has no summoner id".to_owned());
    };
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|error| format!("failed to create LCU HTTP client: {error}"))?;
    let url = format!(
        "{}/lol-collections/v1/inventories/{summoner_id}/champion-mastery",
        lockfile.base_url()
    );
    let response = client
        .get(url)
        .basic_auth("riot", Some(lockfile.password()))
        .send()
        .await
        .map_err(|error| format!("failed to call League Client champion mastery API: {error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "League Client champion mastery API returned HTTP {}",
            response.status()
        ));
    }

    let masteries = response
        .json::<Vec<LcuChampionMastery>>()
        .await
        .map_err(|error| {
            format!("failed to parse League Client champion mastery response: {error}")
        })?;

    let champion_masteries = top_champion_masteries_from_lcu(masteries);

    if champion_masteries.is_empty() {
        return Err("League Client champion mastery API returned no masteries".to_owned());
    }

    summoner.champion_masteries = champion_masteries;
    Ok(())
}

fn top_champion_masteries_from_lcu(
    mut masteries: Vec<LcuChampionMastery>,
) -> Vec<RiotChampionMasteryResponse> {
    masteries.sort_by(|first, second| second.champion_points.cmp(&first.champion_points));

    masteries
        .into_iter()
        .take(5)
        .map(|mastery| RiotChampionMasteryResponse {
            champion_id: mastery.champion_id,
            champion_level: mastery.champion_level,
            champion_points: mastery.champion_points,
        })
        .collect()
}

fn match_history_entry_from_detail(detail: &Value, puuid: &str) -> Option<MatchHistoryEntry> {
    let info = detail.get("info")?;
    let metadata = detail.get("metadata")?;
    let participants = info.get("participants")?.as_array()?;
    let participant = participants
        .iter()
        .find(|participant| participant.get("puuid").and_then(Value::as_str) == Some(puuid))?;
    let match_id = metadata.get("matchId")?.as_str()?.to_owned();

    Some(MatchHistoryEntry {
        assists: participant.get("assists")?.as_u64()?.try_into().ok()?,
        champion_id: participant.get("championId")?.as_u64()?.try_into().ok()?,
        champion_name: participant.get("championName")?.as_str()?.to_owned(),
        deaths: participant.get("deaths")?.as_u64()?.try_into().ok()?,
        duration_seconds: info.get("gameDuration")?.as_u64()?,
        game_created_at: info.get("gameCreation")?.as_i64()?,
        kills: participant.get("kills")?.as_u64()?.try_into().ok()?,
        lp_delta: None,
        match_id,
        queue_id: info.get("queueId")?.as_u64()?.try_into().ok()?,
        role: participant
            .get("teamPosition")
            .and_then(Value::as_str)
            .filter(|role| !role.is_empty())
            .unwrap_or("UNKNOWN")
            .to_owned(),
        win: participant.get("win")?.as_bool()?,
    })
}

fn match_detail_response_from_values(
    detail: &Value,
    timeline: &Value,
) -> Option<MatchDetailResponse> {
    let info = detail.get("info")?;
    let metadata = detail.get("metadata")?;
    let participants = info.get("participants")?.as_array()?;
    let mut blue = Vec::new();
    let mut red = Vec::new();

    for participant in participants {
        let parsed = match_participant_from_value(participant)?;

        if participant.get("teamId")?.as_u64()? == 100 {
            blue.push(parsed);
        } else {
            red.push(parsed);
        }
    }

    let blue_result = team_result(&blue);
    let red_result = team_result(&red);

    Some(MatchDetailResponse {
        duration_seconds: info.get("gameDuration")?.as_u64()?,
        game_created_at: info.get("gameCreation")?.as_i64()?,
        match_id: metadata.get("matchId")?.as_str()?.to_owned(),
        queue_id: info.get("queueId")?.as_u64()?.try_into().ok()?,
        teams: vec![
            MatchTeam {
                participants: blue,
                result: blue_result,
                team_id: 100,
            },
            MatchTeam {
                participants: red,
                result: red_result,
                team_id: 200,
            },
        ],
        timeline: match_timeline_points_from_value(timeline),
    })
}

fn match_participant_from_value(participant: &Value) -> Option<MatchParticipant> {
    let game_name = participant
        .get("riotIdGameName")
        .and_then(Value::as_str)
        .or_else(|| participant.get("summonerName").and_then(Value::as_str))
        .unwrap_or("Unknown");
    let tag_line = participant
        .get("riotIdTagline")
        .and_then(Value::as_str)
        .unwrap_or("");
    let riot_id = if tag_line.is_empty() {
        game_name.to_owned()
    } else {
        format!("{game_name}#{tag_line}")
    };

    Some(MatchParticipant {
        assists: value_u32(participant, "assists"),
        champion_id: value_u32(participant, "championId"),
        champion_level: value_u32(participant, "champLevel"),
        champion_name: participant.get("championName")?.as_str()?.to_owned(),
        cs: value_u32(participant, "totalMinionsKilled")
            + value_u32(participant, "neutralMinionsKilled"),
        deaths: value_u32(participant, "deaths"),
        gold_earned: value_u32(participant, "goldEarned"),
        items: (0..=6)
            .map(|slot| value_u32(participant, &format!("item{slot}")))
            .filter(|item| *item > 0)
            .collect(),
        kills: value_u32(participant, "kills"),
        participant_id: value_u32(participant, "participantId"),
        riot_id,
        summoner_spell_ids: vec![
            value_u32(participant, "summoner1Id"),
            value_u32(participant, "summoner2Id"),
        ],
        team_position: participant
            .get("teamPosition")
            .and_then(Value::as_str)
            .filter(|role| !role.is_empty())
            .unwrap_or("UNKNOWN")
            .to_owned(),
        total_damage_to_champions: value_u32(participant, "totalDamageDealtToChampions"),
        vision_score: value_u32(participant, "visionScore"),
        win: participant.get("win")?.as_bool()?,
    })
}

fn team_result(participants: &[MatchParticipant]) -> String {
    if participants.iter().any(|participant| participant.win) {
        "Victory".to_owned()
    } else {
        "Defeat".to_owned()
    }
}

fn match_timeline_points_from_value(timeline: &Value) -> Vec<MatchTimelinePoint> {
    timeline
        .get("info")
        .and_then(|info| info.get("frames"))
        .and_then(Value::as_array)
        .map(|frames| {
            frames
                .iter()
                .filter_map(match_timeline_point_from_frame)
                .collect()
        })
        .unwrap_or_default()
}

fn match_timeline_point_from_frame(frame: &Value) -> Option<MatchTimelinePoint> {
    let minute = frame.get("timestamp")?.as_u64()? / 60_000;
    let participant_frames = frame.get("participantFrames")?.as_object()?;
    let mut point = MatchTimelinePoint {
        blue_damage: 0,
        blue_gold: 0,
        blue_xp: 0,
        minute: minute.try_into().ok()?,
        red_damage: 0,
        red_gold: 0,
        red_xp: 0,
    };

    for (id, participant_frame) in participant_frames {
        let participant_id = id.parse::<u32>().ok()?;
        let is_blue = participant_id <= 5;
        let damage = participant_frame
            .get("damageStats")
            .map(|stats| value_u32(stats, "totalDamageDoneToChampions"))
            .unwrap_or(0);
        let gold = value_u32(participant_frame, "totalGold");
        let xp = value_u32(participant_frame, "xp");

        if is_blue {
            point.blue_damage += damage;
            point.blue_gold += gold;
            point.blue_xp += xp;
        } else {
            point.red_damage += damage;
            point.red_gold += gold;
            point.red_xp += xp;
        }
    }

    Some(point)
}

fn value_u32(value: &Value, key: &str) -> u32 {
    value
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|number| number.try_into().ok())
        .unwrap_or(0)
}

async fn hydrate_current_summoner_account_identity(
    client: &arkan_core::RiotApiClient,
    summoner: &mut CurrentSummoner,
) {
    let riot_id_input = match (summoner.game_name.as_deref(), summoner.tag_line.as_deref()) {
        (Some(game_name), Some(tag_line)) => format!("{game_name}#{tag_line}"),
        _ => summoner.display_name.clone(),
    };
    let Ok(riot_id) = arkan_core::RiotId::parse(&riot_id_input) else {
        return;
    };

    let Ok(account) = client
        .account_by_riot_id(arkan_core::RegionalRoute::Europe, &riot_id)
        .await
    else {
        return;
    };

    summoner.game_name = Some(account.game_name);
    summoner.tag_line = Some(account.tag_line);
    summoner.puuid = Some(account.puuid);
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            parse_riot_id,
            config_status,
            resolve_riot_account,
            local_database_status,
            match_history,
            sync_champion_sample,
            match_detail,
            league_client_status,
            refresh_champion_role_stats,
            champion_role_stats,
            champion_spell_pairs,
            champion_rune_pages
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Arkan");
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn temp_database_path(test_name: &str) -> PathBuf {
        let unique_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after Unix epoch")
            .as_nanos();

        env::temp_dir().join(format!(
            "arkan-{test_name}-{}-{unique_id}.sqlite3",
            std::process::id()
        ))
    }

    #[test]
    fn persists_current_summoner_to_sqlite() {
        let path = temp_database_path("current-summoner");
        let summoner = CurrentSummoner {
            display_name: "PrincesseMargaux".to_owned(),
            game_name: Some("PrincesseMargaux".to_owned()),
            tag_line: Some("9096".to_owned()),
            puuid: Some("puuid-local".to_owned()),
            summoner_id: Some(123456),
            profile_icon_id: Some(29),
            summoner_level: Some(175),
            champion_masteries: Vec::new(),
        };

        persist_current_summoner_at(&path, &summoner).unwrap();

        let connection = open_database_at(&path).unwrap();
        let player = arkan_core::find_player_by_puuid(&connection, "puuid-local")
            .unwrap()
            .expect("persisted player should be readable");

        assert_eq!(player.game_name, "PrincesseMargaux");
        assert_eq!(player.tag_line, "9096");
        assert_eq!(player.platform_id, "EUW1");
        assert_eq!(player.summoner_id.as_deref(), Some("123456"));
        assert_eq!(player.summoner_level, Some(175));
        assert_eq!(player.profile_icon_id, Some(29));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn builds_current_summoner_from_chat_profile_fallback() {
        let chat_profile = LcuChatMe {
            game_name: Some("PrincesseMargaux".to_owned()),
            game_tag: Some("9096".to_owned()),
            name: None,
            pid: Some("PrincesseMargaux#9096".to_owned()),
            puuid: Some("puuid-chat".to_owned()),
            summoner_id: Some(LcuNumericId::Text("123456".to_owned())),
            icon: Some(LcuNumericId::Number(29)),
        };

        let summoner = current_summoner_from_chat_profile(chat_profile);

        assert_eq!(summoner.display_name, "PrincesseMargaux#9096");
        assert_eq!(summoner.game_name.as_deref(), Some("PrincesseMargaux"));
        assert_eq!(summoner.tag_line.as_deref(), Some("9096"));
        assert_eq!(summoner.puuid.as_deref(), Some("puuid-chat"));
        assert_eq!(summoner.summoner_id, Some(123456));
        assert_eq!(summoner.profile_icon_id, Some(29));
        assert_eq!(summoner.summoner_level, None);
        assert!(summoner.champion_masteries.is_empty());
    }

    #[test]
    fn sorts_lcu_champion_masteries_by_points() {
        let masteries = top_champion_masteries_from_lcu(vec![
            LcuChampionMastery {
                champion_id: 1,
                champion_level: 4,
                champion_points: 20_000,
            },
            LcuChampionMastery {
                champion_id: 2,
                champion_level: 7,
                champion_points: 80_000,
            },
            LcuChampionMastery {
                champion_id: 3,
                champion_level: 5,
                champion_points: 40_000,
            },
            LcuChampionMastery {
                champion_id: 4,
                champion_level: 6,
                champion_points: 60_000,
            },
            LcuChampionMastery {
                champion_id: 5,
                champion_level: 3,
                champion_points: 10_000,
            },
            LcuChampionMastery {
                champion_id: 6,
                champion_level: 2,
                champion_points: 5_000,
            },
        ]);

        assert_eq!(masteries.len(), 5);
        assert_eq!(
            masteries
                .iter()
                .map(|mastery| mastery.champion_id)
                .collect::<Vec<_>>(),
            vec![2, 4, 3, 1, 5]
        );
    }

    #[test]
    fn builds_current_summoner_identity_key_from_stable_identifiers() {
        let mut summoner = CurrentSummoner {
            display_name: "Display".to_owned(),
            game_name: Some("GameName".to_owned()),
            tag_line: Some("EUW".to_owned()),
            puuid: Some("puuid-local".to_owned()),
            summoner_id: Some(123456),
            profile_icon_id: Some(29),
            summoner_level: Some(175),
            champion_masteries: Vec::new(),
        };

        assert_eq!(
            current_summoner_identity_key(&summoner).as_deref(),
            Some("puuid:puuid-local")
        );

        summoner.puuid = None;
        assert_eq!(
            current_summoner_identity_key(&summoner).as_deref(),
            Some("summoner:123456")
        );

        summoner.summoner_id = None;
        assert_eq!(
            current_summoner_identity_key(&summoner).as_deref(),
            Some("riot-id:gamename#euw")
        );
    }

    #[test]
    fn lcu_session_key_changes_with_lockfile_port() {
        let first =
            arkan_core::LeagueClientLockfile::parse("LeagueClient:1234:50344:password:https")
                .unwrap();
        let second =
            arkan_core::LeagueClientLockfile::parse("LeagueClient:1234:50345:password:https")
                .unwrap();
        let lockfile_path = PathBuf::from(r"C:\Riot Games\League of Legends\lockfile");

        assert_ne!(
            lcu_session_key(&lockfile_path, &first),
            lcu_session_key(&lockfile_path, &second)
        );
    }

    #[test]
    fn current_summoner_persist_key_tracks_stored_fields() {
        let mut summoner = CurrentSummoner {
            display_name: "Display".to_owned(),
            game_name: Some("GameName".to_owned()),
            tag_line: Some("EUW".to_owned()),
            puuid: Some("puuid-local".to_owned()),
            summoner_id: Some(123456),
            profile_icon_id: Some(29),
            summoner_level: Some(175),
            champion_masteries: Vec::new(),
        };
        let first_key = current_summoner_persist_key(&summoner).unwrap();

        summoner.summoner_level = Some(176);
        let level_key = current_summoner_persist_key(&summoner).unwrap();

        summoner.summoner_level = Some(175);
        summoner.profile_icon_id = Some(30);
        let icon_key = current_summoner_persist_key(&summoner).unwrap();

        assert_ne!(first_key, level_key);
        assert_ne!(first_key, icon_key);
    }

    #[test]
    fn current_summoner_persist_cache_recognizes_identical_snapshot() {
        let mut summoner = CurrentSummoner {
            display_name: "Display".to_owned(),
            game_name: Some("GameName".to_owned()),
            tag_line: Some("EUW".to_owned()),
            puuid: Some(format!("puuid-cache-{}", std::process::id())),
            summoner_id: Some(123456),
            profile_icon_id: Some(29),
            summoner_level: Some(175),
            champion_masteries: Vec::new(),
        };
        let first_key = current_summoner_persist_key(&summoner).unwrap();

        store_persisted_current_summoner_key(&first_key);
        assert!(recently_persisted_current_summoner_key(&first_key));

        summoner.profile_icon_id = Some(30);
        let changed_key = current_summoner_persist_key(&summoner).unwrap();
        assert!(!recently_persisted_current_summoner_key(&changed_key));
    }

    #[test]
    fn extracts_match_history_entry_for_player() {
        let detail = serde_json::json!({
            "metadata": {
                "matchId": "EUW1_123"
            },
            "info": {
                "gameCreation": 1710000000000_i64,
                "gameDuration": 1820,
                "gameVersion": "16.12.1",
                "queueId": 420,
                "participants": [
                    {
                        "puuid": "other-puuid",
                        "championId": 1,
                        "championName": "Annie",
                        "kills": 1,
                        "deaths": 2,
                        "assists": 3,
                        "teamPosition": "MIDDLE",
                        "win": false
                    },
                    {
                        "puuid": "player-puuid",
                        "championId": 166,
                        "championName": "Akshan",
                        "kills": 12,
                        "deaths": 4,
                        "assists": 8,
                        "teamPosition": "BOTTOM",
                        "totalMinionsKilled": 220,
                        "neutralMinionsKilled": 21,
                        "goldEarned": 15400,
                        "visionScore": 22,
                        "win": true
                    }
                ]
            }
        });

        let entry = match_history_entry_from_detail(&detail, "player-puuid").unwrap();

        assert_eq!(entry.match_id, "EUW1_123");
        assert_eq!(entry.champion_id, 166);
        assert_eq!(entry.champion_name, "Akshan");
        assert_eq!(entry.lp_delta, None);
        assert_eq!(entry.kills, 12);
        assert_eq!(entry.deaths, 4);
        assert_eq!(entry.assists, 8);
        assert_eq!(entry.role, "BOTTOM");
        assert!(entry.win);

        let match_record =
            match_record_from_detail(&detail, arkan_core::RegionalRoute::Europe).unwrap();
        let player_match =
            player_match_record_from_detail(&detail, "player-puuid", &entry).unwrap();

        assert_eq!(match_record.game_version.as_deref(), Some("16.12.1"));
        assert!(match_record.raw_json.contains("EUW1_123"));
        assert_eq!(player_match.total_cs, 241);
        assert_eq!(player_match.gold_earned, 15_400);
        assert_eq!(player_match.vision_score, 22);
    }

    #[test]
    fn extracts_match_detail_with_team_timeline() {
        let detail = serde_json::json!({
            "metadata": {
                "matchId": "EUW1_456"
            },
            "info": {
                "gameCreation": 1710000000000_i64,
                "gameDuration": 1820,
                "queueId": 420,
                "participants": [
                    {
                        "participantId": 1,
                        "teamId": 100,
                        "riotIdGameName": "BlueCarry",
                        "riotIdTagline": "EUW",
                        "championId": 22,
                        "championName": "Ashe",
                        "champLevel": 15,
                        "kills": 10,
                        "deaths": 2,
                        "assists": 7,
                        "teamPosition": "BOTTOM",
                        "totalDamageDealtToChampions": 23000,
                        "goldEarned": 14500,
                        "totalMinionsKilled": 220,
                        "neutralMinionsKilled": 12,
                        "visionScore": 18,
                        "item0": 6672,
                        "item1": 3006,
                        "item2": 0,
                        "item3": 3031,
                        "item4": 0,
                        "item5": 0,
                        "item6": 3363,
                        "summoner1Id": 4,
                        "summoner2Id": 7,
                        "win": true
                    },
                    {
                        "participantId": 6,
                        "teamId": 200,
                        "riotIdGameName": "RedCarry",
                        "riotIdTagline": "EUW",
                        "championId": 51,
                        "championName": "Caitlyn",
                        "champLevel": 14,
                        "kills": 5,
                        "deaths": 6,
                        "assists": 4,
                        "teamPosition": "BOTTOM",
                        "totalDamageDealtToChampions": 18000,
                        "goldEarned": 12100,
                        "totalMinionsKilled": 205,
                        "neutralMinionsKilled": 0,
                        "visionScore": 12,
                        "item0": 6671,
                        "item1": 3006,
                        "item2": 3031,
                        "item3": 0,
                        "item4": 0,
                        "item5": 0,
                        "item6": 3363,
                        "summoner1Id": 4,
                        "summoner2Id": 21,
                        "win": false
                    }
                ]
            }
        });
        let timeline = serde_json::json!({
            "info": {
                "frames": [
                    {
                        "timestamp": 60000,
                        "participantFrames": {
                            "1": {
                                "totalGold": 520,
                                "xp": 280,
                                "damageStats": { "totalDamageDoneToChampions": 120 }
                            },
                            "6": {
                                "totalGold": 500,
                                "xp": 260,
                                "damageStats": { "totalDamageDoneToChampions": 90 }
                            }
                        }
                    }
                ]
            }
        });

        let response = match_detail_response_from_values(&detail, &timeline).unwrap();

        assert_eq!(response.match_id, "EUW1_456");
        assert_eq!(response.teams[0].result, "Victory");
        assert_eq!(response.teams[0].participants[0].riot_id, "BlueCarry#EUW");
        assert_eq!(response.teams[0].participants[0].cs, 232);
        assert_eq!(
            response.teams[0].participants[0].items,
            vec![6672, 3006, 3031, 3363]
        );
        assert_eq!(response.timeline[0].minute, 1);
        assert_eq!(response.timeline[0].blue_gold, 520);
        assert_eq!(response.timeline[0].red_damage, 90);
    }
}
