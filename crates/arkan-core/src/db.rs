use std::error::Error;
use std::fmt::{self, Display, Formatter};

use rusqlite::{Connection, OptionalExtension, params};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerRecord {
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
    pub platform_id: String,
    pub summoner_id: Option<String>,
    pub account_id: Option<String>,
    pub summoner_level: Option<u32>,
    pub profile_icon_id: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchRecord {
    pub match_id: String,
    pub regional_route: String,
    pub game_creation: Option<i64>,
    pub game_duration: Option<u64>,
    pub queue_id: Option<u32>,
    pub game_version: Option<String>,
    pub raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerMatchRecord {
    pub puuid: String,
    pub match_id: String,
    pub champion_id: u32,
    pub champion_name: Option<String>,
    pub team_position: Option<String>,
    pub win: bool,
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    pub total_cs: u32,
    pub gold_earned: u32,
    pub vision_score: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChampionRoleStats {
    pub champion_id: u32,
    pub champion_key: String,
    pub champion_name: String,
    pub role: String,
    pub patch: String,
    pub platform_id: String,
    pub queue_id: u32,
    pub tier: Option<String>,
    pub sample_size: u32,
    pub wins: u32,
    pub win_rate: f64,
    pub pick_rate: f64,
    pub source: String,
}

pub fn migrate(connection: &mut Connection) -> Result<(), DbError> {
    let transaction = connection.transaction()?;

    transaction.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS players (
            puuid TEXT PRIMARY KEY,
            game_name TEXT NOT NULL,
            tag_line TEXT NOT NULL,
            platform_id TEXT NOT NULL,
            summoner_id TEXT,
            account_id TEXT,
            summoner_level INTEGER,
            profile_icon_id INTEGER,
            last_refreshed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_players_riot_id
            ON players(game_name, tag_line, platform_id);

        CREATE TABLE IF NOT EXISTS player_ranks (
            id TEXT PRIMARY KEY,
            puuid TEXT NOT NULL,
            queue_type TEXT NOT NULL,
            tier TEXT,
            rank TEXT,
            league_points INTEGER NOT NULL DEFAULT 0,
            wins INTEGER NOT NULL DEFAULT 0,
            losses INTEGER NOT NULL DEFAULT 0,
            fetched_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (puuid) REFERENCES players(puuid) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_player_ranks_puuid_queue
            ON player_ranks(puuid, queue_type, fetched_at DESC);

        CREATE TABLE IF NOT EXISTS matches (
            match_id TEXT PRIMARY KEY,
            regional_route TEXT NOT NULL,
            game_creation INTEGER,
            game_duration INTEGER,
            queue_id INTEGER,
            game_version TEXT,
            raw_json TEXT NOT NULL,
            fetched_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX IF NOT EXISTS idx_matches_game_creation
            ON matches(game_creation DESC);

        CREATE TABLE IF NOT EXISTS player_matches (
            id TEXT PRIMARY KEY,
            puuid TEXT NOT NULL,
            match_id TEXT NOT NULL,
            champion_id INTEGER NOT NULL,
            champion_name TEXT,
            team_position TEXT,
            win INTEGER NOT NULL,
            kills INTEGER NOT NULL DEFAULT 0,
            deaths INTEGER NOT NULL DEFAULT 0,
            assists INTEGER NOT NULL DEFAULT 0,
            total_cs INTEGER NOT NULL DEFAULT 0,
            gold_earned INTEGER NOT NULL DEFAULT 0,
            vision_score INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (puuid) REFERENCES players(puuid) ON DELETE CASCADE,
            FOREIGN KEY (match_id) REFERENCES matches(match_id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_player_matches_puuid
            ON player_matches(puuid);

        CREATE INDEX IF NOT EXISTS idx_player_matches_match_id
            ON player_matches(match_id);

        CREATE TABLE IF NOT EXISTS champion_role_stats (
            id TEXT PRIMARY KEY,
            champion_id INTEGER NOT NULL,
            champion_key TEXT NOT NULL,
            champion_name TEXT NOT NULL,
            role TEXT NOT NULL,
            patch TEXT NOT NULL,
            platform_id TEXT NOT NULL,
            queue_id INTEGER NOT NULL,
            tier TEXT,
            sample_size INTEGER NOT NULL,
            wins INTEGER NOT NULL,
            win_rate REAL NOT NULL,
            pick_rate REAL NOT NULL,
            source TEXT NOT NULL,
            collected_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(champion_id, role, patch, platform_id, queue_id, tier)
        );

        CREATE INDEX IF NOT EXISTS idx_champion_role_stats_lookup
            ON champion_role_stats(champion_id, role, patch, platform_id, queue_id);

        CREATE TABLE IF NOT EXISTS champion_skill_orders (
            id TEXT PRIMARY KEY,
            stats_id TEXT NOT NULL,
            skill_order TEXT NOT NULL,
            games INTEGER NOT NULL,
            wins INTEGER NOT NULL,
            win_rate REAL NOT NULL,
            FOREIGN KEY (stats_id) REFERENCES champion_role_stats(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_champion_skill_orders_stats
            ON champion_skill_orders(stats_id, games DESC);

        CREATE TABLE IF NOT EXISTS champion_item_builds (
            id TEXT PRIMARY KEY,
            stats_id TEXT NOT NULL,
            build_type TEXT NOT NULL,
            item_ids TEXT NOT NULL,
            games INTEGER NOT NULL,
            wins INTEGER NOT NULL,
            win_rate REAL NOT NULL,
            FOREIGN KEY (stats_id) REFERENCES champion_role_stats(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_champion_item_builds_stats
            ON champion_item_builds(stats_id, build_type, games DESC);

        INSERT OR IGNORE INTO schema_migrations(version) VALUES (1);
        INSERT OR IGNORE INTO schema_migrations(version) VALUES (2);
        "#,
    )?;

    transaction.commit()?;
    Ok(())
}

pub fn schema_version(connection: &Connection) -> Result<u32, DbError> {
    connection
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get::<_, u32>(0),
        )
        .map_err(DbError::Sqlite)
}

pub fn upsert_player(connection: &Connection, player: &PlayerRecord) -> Result<(), DbError> {
    connection.execute(
        r#"
        INSERT INTO players (
            puuid,
            game_name,
            tag_line,
            platform_id,
            summoner_id,
            account_id,
            summoner_level,
            profile_icon_id,
            last_refreshed_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, CURRENT_TIMESTAMP)
        ON CONFLICT(puuid) DO UPDATE SET
            game_name = excluded.game_name,
            tag_line = excluded.tag_line,
            platform_id = excluded.platform_id,
            summoner_id = excluded.summoner_id,
            account_id = excluded.account_id,
            summoner_level = excluded.summoner_level,
            profile_icon_id = excluded.profile_icon_id,
            last_refreshed_at = CURRENT_TIMESTAMP
        "#,
        params![
            player.puuid,
            player.game_name,
            player.tag_line,
            player.platform_id,
            player.summoner_id,
            player.account_id,
            player.summoner_level,
            player.profile_icon_id
        ],
    )?;

    Ok(())
}

pub fn find_player_by_puuid(
    connection: &Connection,
    puuid: &str,
) -> Result<Option<PlayerRecord>, DbError> {
    connection
        .query_row(
            r#"
            SELECT
                puuid,
                game_name,
                tag_line,
                platform_id,
                summoner_id,
                account_id,
                summoner_level,
                profile_icon_id
            FROM players
            WHERE puuid = ?1
            "#,
            [puuid],
            player_from_row,
        )
        .optional()
        .map_err(DbError::Sqlite)
}

pub fn upsert_match(connection: &Connection, record: &MatchRecord) -> Result<(), DbError> {
    connection.execute(
        r#"
        INSERT INTO matches (
            match_id,
            regional_route,
            game_creation,
            game_duration,
            queue_id,
            game_version,
            raw_json,
            fetched_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, CURRENT_TIMESTAMP)
        ON CONFLICT(match_id) DO UPDATE SET
            regional_route = excluded.regional_route,
            game_creation = excluded.game_creation,
            game_duration = excluded.game_duration,
            queue_id = excluded.queue_id,
            game_version = excluded.game_version,
            raw_json = excluded.raw_json,
            fetched_at = CURRENT_TIMESTAMP
        "#,
        params![
            record.match_id,
            record.regional_route,
            record.game_creation,
            record.game_duration,
            record.queue_id,
            record.game_version,
            record.raw_json
        ],
    )?;

    Ok(())
}

pub fn upsert_player_match(
    connection: &Connection,
    record: &PlayerMatchRecord,
) -> Result<(), DbError> {
    let id = format!("{}:{}", record.puuid, record.match_id);

    connection.execute(
        r#"
        INSERT INTO player_matches (
            id,
            puuid,
            match_id,
            champion_id,
            champion_name,
            team_position,
            win,
            kills,
            deaths,
            assists,
            total_cs,
            gold_earned,
            vision_score
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
        ON CONFLICT(id) DO UPDATE SET
            champion_id = excluded.champion_id,
            champion_name = excluded.champion_name,
            team_position = excluded.team_position,
            win = excluded.win,
            kills = excluded.kills,
            deaths = excluded.deaths,
            assists = excluded.assists,
            total_cs = excluded.total_cs,
            gold_earned = excluded.gold_earned,
            vision_score = excluded.vision_score
        "#,
        params![
            id,
            record.puuid,
            record.match_id,
            record.champion_id,
            record.champion_name,
            record.team_position,
            record.win,
            record.kills,
            record.deaths,
            record.assists,
            record.total_cs,
            record.gold_earned,
            record.vision_score
        ],
    )?;

    Ok(())
}

pub fn upsert_champion_role_stats(
    connection: &Connection,
    stats: &ChampionRoleStats,
) -> Result<(), DbError> {
    let stats_id = champion_role_stats_id(stats);

    connection.execute(
        r#"
        INSERT INTO champion_role_stats (
            id,
            champion_id,
            champion_key,
            champion_name,
            role,
            patch,
            platform_id,
            queue_id,
            tier,
            sample_size,
            wins,
            win_rate,
            pick_rate,
            source,
            collected_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, CURRENT_TIMESTAMP)
        ON CONFLICT(id) DO UPDATE SET
            champion_key = excluded.champion_key,
            champion_name = excluded.champion_name,
            sample_size = excluded.sample_size,
            wins = excluded.wins,
            win_rate = excluded.win_rate,
            pick_rate = excluded.pick_rate,
            source = excluded.source,
            collected_at = CURRENT_TIMESTAMP
        "#,
        params![
            stats_id,
            stats.champion_id,
            stats.champion_key,
            stats.champion_name,
            stats.role,
            stats.patch,
            stats.platform_id,
            stats.queue_id,
            stats.tier,
            stats.sample_size,
            stats.wins,
            stats.win_rate,
            stats.pick_rate,
            stats.source
        ],
    )?;

    Ok(())
}

pub fn find_champion_role_stats(
    connection: &Connection,
    champion_id: u32,
    role: &str,
    patch: &str,
    platform_id: &str,
    queue_id: u32,
    tier: Option<&str>,
) -> Result<Option<ChampionRoleStats>, DbError> {
    connection
        .query_row(
            r#"
            SELECT
                champion_id,
                champion_key,
                champion_name,
                role,
                patch,
                platform_id,
                queue_id,
                tier,
                sample_size,
                wins,
                win_rate,
                pick_rate,
                source
            FROM champion_role_stats
            WHERE champion_id = ?1
                AND role = ?2
                AND patch = ?3
                AND platform_id = ?4
                AND queue_id = ?5
                AND (tier IS ?6 OR tier = ?6)
            "#,
            params![champion_id, role, patch, platform_id, queue_id, tier],
            champion_role_stats_from_row,
        )
        .optional()
        .map_err(DbError::Sqlite)
}

fn champion_role_stats_id(stats: &ChampionRoleStats) -> String {
    format!(
        "{}:{}:{}:{}:{}:{}",
        stats.patch,
        stats.platform_id,
        stats.queue_id,
        stats.tier.as_deref().unwrap_or("ALL"),
        stats.role,
        stats.champion_id
    )
}

fn champion_role_stats_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ChampionRoleStats> {
    Ok(ChampionRoleStats {
        champion_id: row.get(0)?,
        champion_key: row.get(1)?,
        champion_name: row.get(2)?,
        role: row.get(3)?,
        patch: row.get(4)?,
        platform_id: row.get(5)?,
        queue_id: row.get(6)?,
        tier: row.get(7)?,
        sample_size: row.get(8)?,
        wins: row.get(9)?,
        win_rate: row.get(10)?,
        pick_rate: row.get(11)?,
        source: row.get(12)?,
    })
}

fn player_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PlayerRecord> {
    Ok(PlayerRecord {
        puuid: row.get(0)?,
        game_name: row.get(1)?,
        tag_line: row.get(2)?,
        platform_id: row.get(3)?,
        summoner_id: row.get(4)?,
        account_id: row.get(5)?,
        summoner_level: row.get(6)?,
        profile_icon_id: row.get(7)?,
    })
}

#[derive(Debug)]
pub enum DbError {
    Sqlite(rusqlite::Error),
}

impl Display for DbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "SQLite error: {error}"),
        }
    }
}

impl Error for DbError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
        }
    }
}

impl From<rusqlite::Error> for DbError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sqlite(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn migrated_connection() -> Connection {
        let mut connection = Connection::open_in_memory().unwrap();
        migrate(&mut connection).unwrap();
        connection
    }

    #[test]
    fn migrations_create_schema_version() {
        let connection = migrated_connection();

        assert_eq!(schema_version(&connection).unwrap(), 2);
    }

    #[test]
    fn migrations_are_idempotent() {
        let mut connection = migrated_connection();
        migrate(&mut connection).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 2);
    }

    #[test]
    fn upserts_and_reads_player() {
        let connection = migrated_connection();
        let player = PlayerRecord {
            puuid: "puuid-1".to_owned(),
            game_name: "PrincesseMargaux".to_owned(),
            tag_line: "9096".to_owned(),
            platform_id: "EUW1".to_owned(),
            summoner_id: Some("summoner-1".to_owned()),
            account_id: Some("account-1".to_owned()),
            summoner_level: Some(175),
            profile_icon_id: Some(588),
        };

        upsert_player(&connection, &player).unwrap();

        assert_eq!(
            find_player_by_puuid(&connection, "puuid-1").unwrap(),
            Some(player)
        );
    }

    #[test]
    fn upsert_player_updates_existing_row() {
        let connection = migrated_connection();
        let mut player = PlayerRecord {
            puuid: "puuid-1".to_owned(),
            game_name: "OldName".to_owned(),
            tag_line: "EUW".to_owned(),
            platform_id: "EUW1".to_owned(),
            summoner_id: None,
            account_id: None,
            summoner_level: Some(10),
            profile_icon_id: Some(1),
        };

        upsert_player(&connection, &player).unwrap();
        player.game_name = "NewName".to_owned();
        player.summoner_level = Some(11);
        upsert_player(&connection, &player).unwrap();

        assert_eq!(
            find_player_by_puuid(&connection, "puuid-1").unwrap(),
            Some(player)
        );
    }

    #[test]
    fn upserts_and_reads_champion_role_stats() {
        let connection = migrated_connection();
        let mut stats = ChampionRoleStats {
            champion_id: 29,
            champion_key: "Twitch".to_owned(),
            champion_name: "Twitch".to_owned(),
            role: "BOTTOM".to_owned(),
            patch: "16.12".to_owned(),
            platform_id: "EUW1".to_owned(),
            queue_id: 420,
            tier: Some("EMERALD_PLUS".to_owned()),
            sample_size: 100,
            wins: 53,
            win_rate: 53.0,
            pick_rate: 4.2,
            source: "riot-match-v5".to_owned(),
        };

        upsert_champion_role_stats(&connection, &stats).unwrap();
        stats.sample_size = 120;
        stats.wins = 66;
        stats.win_rate = 55.0;
        upsert_champion_role_stats(&connection, &stats).unwrap();

        assert_eq!(
            find_champion_role_stats(
                &connection,
                29,
                "BOTTOM",
                "16.12",
                "EUW1",
                420,
                Some("EMERALD_PLUS")
            )
            .unwrap(),
            Some(stats)
        );
    }

    #[test]
    fn upserts_match_and_player_match() {
        let connection = migrated_connection();
        let player = PlayerRecord {
            puuid: "puuid-1".to_owned(),
            game_name: "Player".to_owned(),
            tag_line: "EUW".to_owned(),
            platform_id: "EUW1".to_owned(),
            summoner_id: None,
            account_id: None,
            summoner_level: None,
            profile_icon_id: None,
        };
        let match_record = MatchRecord {
            match_id: "EUW1_123".to_owned(),
            regional_route: "europe".to_owned(),
            game_creation: Some(1_710_000_000_000),
            game_duration: Some(1_820),
            queue_id: Some(420),
            game_version: Some("16.12.1".to_owned()),
            raw_json: "{\"metadata\":{}}".to_owned(),
        };
        let player_match = PlayerMatchRecord {
            puuid: player.puuid.clone(),
            match_id: match_record.match_id.clone(),
            champion_id: 166,
            champion_name: Some("Akshan".to_owned()),
            team_position: Some("MIDDLE".to_owned()),
            win: true,
            kills: 12,
            deaths: 4,
            assists: 8,
            total_cs: 241,
            gold_earned: 15_400,
            vision_score: 22,
        };

        upsert_player(&connection, &player).unwrap();
        upsert_match(&connection, &match_record).unwrap();
        upsert_player_match(&connection, &player_match).unwrap();

        let stored: (String, String, u32, u32, u32) = connection
            .query_row(
                "SELECT m.raw_json, pm.champion_name, pm.total_cs, pm.gold_earned, pm.vision_score FROM matches m JOIN player_matches pm ON pm.match_id = m.match_id WHERE m.match_id = ?1",
                [&match_record.match_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .unwrap();

        assert_eq!(
            stored,
            (match_record.raw_json, "Akshan".to_owned(), 241, 15_400, 22)
        );
    }
}
