#[tauri::command]
fn parse_riot_id(input: &str) -> Result<String, String> {
    arkan_core::RiotId::parse(input)
        .map(|riot_id| riot_id.to_string())
        .map_err(|error| error.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![parse_riot_id])
        .run(tauri::generate_context!())
        .expect("failed to run Arkan");
}

