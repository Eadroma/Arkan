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

        INSERT OR IGNORE INTO schema_migrations(version) VALUES (1);
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

        assert_eq!(schema_version(&connection).unwrap(), 1);
    }

    #[test]
    fn migrations_are_idempotent() {
        let mut connection = migrated_connection();
        migrate(&mut connection).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 1);
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
}
