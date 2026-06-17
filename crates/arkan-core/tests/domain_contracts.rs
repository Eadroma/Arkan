use arkan_core::{
    AppConfig, LeagueClientLockfile, PlatformRoute, PlayerRecord, RegionalRoute, RiotId,
    account_by_riot_id_url, find_player_by_puuid, migrate, schema_version, upsert_player,
};

#[test]
fn riot_id_display_round_trips_after_normalization() {
    let riot_id = RiotId::parse("  PrincesseMargaux  #  9096  ").unwrap();

    assert_eq!(riot_id.game_name(), "PrincesseMargaux");
    assert_eq!(riot_id.tag_line(), "9096");
    assert_eq!(riot_id.to_string(), "PrincesseMargaux#9096");
}

#[test]
fn league_client_lockfile_keeps_credentials_private_by_default() {
    let lockfile = LeagueClientLockfile::parse("LeagueClient:4120:65358:secret:https").unwrap();

    assert_eq!(lockfile.process_name(), "LeagueClient");
    assert_eq!(lockfile.process_id(), 4120);
    assert_eq!(lockfile.port(), 65358);
    assert_eq!(lockfile.protocol().to_string(), "https");
    assert_eq!(lockfile.base_url(), "https://127.0.0.1:65358");
}

#[test]
fn platform_routes_choose_the_match_v5_regional_route() {
    assert_eq!(PlatformRoute::Euw1.regional_route(), RegionalRoute::Europe);
    assert_eq!(PlatformRoute::Na1.regional_route(), RegionalRoute::Americas);
    assert_eq!(PlatformRoute::Kr.regional_route(), RegionalRoute::Asia);
    assert_eq!(PlatformRoute::Sg2.regional_route(), RegionalRoute::Sea);
}

#[test]
fn app_config_defaults_are_good_for_current_project_region() {
    let config = AppConfig::from_values(None, None, None).unwrap();

    assert_eq!(config.default_platform(), PlatformRoute::Euw1);
    assert_eq!(config.default_language(), "fr_FR");
    assert!(!config.has_riot_api_key());
}

#[test]
fn riot_account_url_uses_region_derived_from_platform() {
    let riot_id = RiotId::parse("PrincesseMargaux#9096").unwrap();
    let platform = PlatformRoute::Euw1;

    assert_eq!(
        account_by_riot_id_url(platform.regional_route(), &riot_id),
        "https://europe.api.riotgames.com/riot/account/v1/accounts/by-riot-id/PrincesseMargaux/9096"
    );
}

#[test]
fn sqlite_schema_can_store_detected_player_identity() {
    let mut connection = rusqlite::Connection::open_in_memory().unwrap();
    migrate(&mut connection).unwrap();

    let player = PlayerRecord {
        puuid: "puuid-local".to_owned(),
        game_name: "PrincesseMargaux".to_owned(),
        tag_line: "9096".to_owned(),
        platform_id: "EUW1".to_owned(),
        summoner_id: None,
        account_id: None,
        summoner_level: Some(175),
        profile_icon_id: Some(588),
    };

    upsert_player(&connection, &player).unwrap();

    assert_eq!(schema_version(&connection).unwrap(), 1);
    assert_eq!(
        find_player_by_puuid(&connection, "puuid-local").unwrap(),
        Some(player)
    );
}
