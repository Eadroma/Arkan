use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value;

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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RiotChampionMasteryResponse {
    champion_id: u32,
    champion_level: u32,
    champion_points: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MatchHistoryEntry {
    assists: u32,
    champion_name: String,
    deaths: u32,
    duration_seconds: u64,
    game_created_at: i64,
    kills: u32,
    match_id: String,
    queue_id: u32,
    role: String,
    win: bool,
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
async fn match_history(input: &str, platform: &str) -> Result<Vec<MatchHistoryEntry>, String> {
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
    let match_ids = client
        .match_ids_by_puuid(platform.regional_route(), &account.puuid, 0, 5)
        .await
        .map_err(|error| error.to_string())?;
    let mut entries = Vec::new();

    for match_id in match_ids {
        let detail = client
            .match_by_id(platform.regional_route(), &match_id)
            .await
            .map_err(|error| error.to_string())?;

        if let Some(entry) = match_history_entry_from_detail(&detail, &account.puuid) {
            entries.push(entry);
        }
    }

    Ok(entries)
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

#[derive(Debug, Serialize)]
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
            hydrate_current_summoner_champion_pool(&lockfile, &mut summoner).await;
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

    arkan_core::upsert_player(&connection, &player).map_err(|error| error.to_string())
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
        champion_name: participant.get("championName")?.as_str()?.to_owned(),
        deaths: participant.get("deaths")?.as_u64()?.try_into().ok()?,
        duration_seconds: info.get("gameDuration")?.as_u64()?,
        game_created_at: info.get("gameCreation")?.as_i64()?,
        kills: participant.get("kills")?.as_u64()?.try_into().ok()?,
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
            league_client_status
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
    fn extracts_match_history_entry_for_player() {
        let detail = serde_json::json!({
            "metadata": {
                "matchId": "EUW1_123"
            },
            "info": {
                "gameCreation": 1710000000000_i64,
                "gameDuration": 1820,
                "queueId": 420,
                "participants": [
                    {
                        "puuid": "other-puuid",
                        "championName": "Annie",
                        "kills": 1,
                        "deaths": 2,
                        "assists": 3,
                        "teamPosition": "MIDDLE",
                        "win": false
                    },
                    {
                        "puuid": "player-puuid",
                        "championName": "Akshan",
                        "kills": 12,
                        "deaths": 4,
                        "assists": 8,
                        "teamPosition": "BOTTOM",
                        "win": true
                    }
                ]
            }
        });

        let entry = match_history_entry_from_detail(&detail, "player-puuid").unwrap();

        assert_eq!(entry.match_id, "EUW1_123");
        assert_eq!(entry.champion_name, "Akshan");
        assert_eq!(entry.kills, 12);
        assert_eq!(entry.deaths, 4);
        assert_eq!(entry.assists, 8);
        assert_eq!(entry.role, "BOTTOM");
        assert!(entry.win);
    }
}
