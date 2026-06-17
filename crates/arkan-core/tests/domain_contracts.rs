use arkan_core::{
    AppConfig, ChampionRoleStats, LeagueClientLockfile, PlatformRoute, PlayerRecord, RegionalRoute,
    RiotId, account_by_riot_id_url, champion_mastery_top_url, find_champion_role_stats,
    find_player_by_puuid, match_by_id_url, match_ids_by_puuid_url, match_timeline_by_id_url,
    migrate, schema_version, summoner_by_puuid_url, upsert_champion_role_stats, upsert_player,
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
fn riot_summoner_url_uses_platform_route() {
    assert_eq!(
        summoner_by_puuid_url(PlatformRoute::Euw1, "puuid-local"),
        "https://euw1.api.riotgames.com/lol/summoner/v4/summoners/by-puuid/puuid-local"
    );
}

#[test]
fn champion_mastery_top_url_uses_platform_route() {
    assert_eq!(
        champion_mastery_top_url(PlatformRoute::Euw1, "puuid-local", 5),
        "https://euw1.api.riotgames.com/lol/champion-mastery/v4/champion-masteries/by-puuid/puuid-local/top?count=5"
    );
}

#[test]
fn match_v5_urls_use_regional_route() {
    assert_eq!(
        match_ids_by_puuid_url(RegionalRoute::Europe, "puuid-local", 0, 20),
        "https://europe.api.riotgames.com/lol/match/v5/matches/by-puuid/puuid-local/ids?start=0&count=20"
    );
    assert_eq!(
        match_by_id_url(RegionalRoute::Europe, "EUW1_123"),
        "https://europe.api.riotgames.com/lol/match/v5/matches/EUW1_123"
    );
    assert_eq!(
        match_timeline_by_id_url(RegionalRoute::Europe, "EUW1_123"),
        "https://europe.api.riotgames.com/lol/match/v5/matches/EUW1_123/timeline"
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

    assert_eq!(schema_version(&connection).unwrap(), 2);
    assert_eq!(
        find_player_by_puuid(&connection, "puuid-local").unwrap(),
        Some(player)
    );
}

#[test]
fn sqlite_schema_can_store_champion_role_aggregates() {
    let mut connection = rusqlite::Connection::open_in_memory().unwrap();
    migrate(&mut connection).unwrap();

    let stats = ChampionRoleStats {
        champion_id: 29,
        champion_key: "Twitch".to_owned(),
        champion_name: "Twitch".to_owned(),
        role: "BOTTOM".to_owned(),
        patch: "16.12".to_owned(),
        platform_id: "EUW1".to_owned(),
        queue_id: 420,
        tier: None,
        sample_size: 80,
        wins: 42,
        win_rate: 52.5,
        pick_rate: 3.9,
        source: "riot-match-v5".to_owned(),
    };

    upsert_champion_role_stats(&connection, &stats).unwrap();

    assert_eq!(
        find_champion_role_stats(&connection, 29, "BOTTOM", "16.12", "EUW1", 420, None).unwrap(),
        Some(stats)
    );
}
