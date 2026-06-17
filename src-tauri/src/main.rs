use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

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

    Ok(RiotAccountResponse {
        puuid: account.puuid,
        game_name: account.game_name,
        tag_line: account.tag_line,
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
        Ok(summoner) => {
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
    let url = format!("{}/lol-summoner/v1/current-summoner", lockfile.base_url());
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|error| format!("failed to create LCU HTTP client: {error}"))?;

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
    })
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            parse_riot_id,
            config_status,
            resolve_riot_account,
            local_database_status,
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
}
