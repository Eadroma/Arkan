use std::env;
use std::fs;
use std::path::PathBuf;

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
struct LeagueClientStatus {
    detected: bool,
    connected: bool,
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
        Ok(summoner) => LeagueClientStatus {
            detected: true,
            connected: true,
            lockfile_path: Some(lockfile_path.display().to_string()),
            port: Some(lockfile.port()),
            protocol: Some(lockfile.protocol().to_string()),
            summoner: Some(summoner),
            error: None,
        },
        Err(error) => LeagueClientStatus {
            detected: true,
            connected: false,
            lockfile_path: Some(lockfile_path.display().to_string()),
            port: Some(lockfile.port()),
            protocol: Some(lockfile.protocol().to_string()),
            summoner: None,
            error: Some(error),
        },
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
            league_client_status
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Arkan");
}
