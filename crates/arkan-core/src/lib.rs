pub mod config;
pub mod db;
pub mod lcu;
pub mod riot_api;
pub mod riot_id;
pub mod routing;

pub use config::{AppConfig, ConfigError};
pub use db::{
    ChampionRoleStats, ChampionRunePageStats, ChampionSpellPairStats, DbError, MatchRecord,
    PlayerMatchRecord, PlayerRecord, find_champion_role_stats,
    find_champion_role_stats_by_champion, find_local_champion_rune_pages,
    find_local_champion_spell_pairs, find_player_by_puuid, migrate,
    refresh_local_champion_role_stats, schema_version, upsert_champion_role_stats, upsert_match,
    upsert_player, upsert_player_match,
};
pub use lcu::{LeagueClientLockfile, LeagueClientProtocol, ParseLockfileError};
pub use riot_api::{
    RiotAccount, RiotApiClient, RiotApiError, RiotChampionMastery, RiotLeagueEntry, RiotLeagueList,
    RiotSummoner, RiotTopLeagueTier, account_by_riot_id_url, champion_mastery_top_url,
    match_by_id_url, match_ids_by_puuid_url, match_timeline_by_id_url, summoner_by_id_url,
    summoner_by_puuid_url, top_league_url,
};
pub use riot_id::{ParseRiotIdError, RiotId};
pub use routing::{PlatformRoute, RegionalRoute};
