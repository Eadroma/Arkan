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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![parse_riot_id, config_status])
        .run(tauri::generate_context!())
        .expect("failed to run Arkan");
}
