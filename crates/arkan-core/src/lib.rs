pub mod config;
pub mod db;
pub mod lcu;
pub mod riot_api;
pub mod riot_id;
pub mod routing;

pub use config::{AppConfig, ConfigError};
pub use db::{
    ChampionRoleStats, DbError, PlayerRecord, find_champion_role_stats, find_player_by_puuid,
    migrate, schema_version, upsert_champion_role_stats, upsert_player,
};
pub use lcu::{LeagueClientLockfile, LeagueClientProtocol, ParseLockfileError};
pub use riot_api::{
    RiotAccount, RiotApiClient, RiotApiError, RiotChampionMastery, RiotSummoner,
    account_by_riot_id_url, champion_mastery_top_url, match_by_id_url, match_ids_by_puuid_url,
    match_timeline_by_id_url, summoner_by_puuid_url,
};
pub use riot_id::{ParseRiotIdError, RiotId};
pub use routing::{PlatformRoute, RegionalRoute};
