use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiotId {
    game_name: String,
    tag_line: String,
}

impl RiotId {
    pub fn parse(input: &str) -> Result<Self, ParseRiotIdError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Err(ParseRiotIdError::Empty);
        }

        let (game_name, tag_line) = trimmed
            .split_once('#')
            .ok_or(ParseRiotIdError::MissingSeparator)?;

        let game_name = game_name.trim();
        let tag_line = tag_line.trim();

        if game_name.is_empty() {
            return Err(ParseRiotIdError::MissingGameName);
        }

        if tag_line.is_empty() {
            return Err(ParseRiotIdError::MissingTagLine);
        }

        if tag_line.contains('#') {
            return Err(ParseRiotIdError::TooManySeparators);
        }

        Ok(Self {
            game_name: game_name.to_owned(),
            tag_line: tag_line.to_owned(),
        })
    }

    pub fn game_name(&self) -> &str {
        &self.game_name
    }

    pub fn tag_line(&self) -> &str {
        &self.tag_line
    }
}

impl Display for RiotId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{}", self.game_name, self.tag_line)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseRiotIdError {
    Empty,
    MissingSeparator,
    MissingGameName,
    MissingTagLine,
    TooManySeparators,
}

impl Display for ParseRiotIdError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("Riot ID is empty"),
            Self::MissingSeparator => f.write_str("Riot ID must use the GameName#TAG format"),
            Self::MissingGameName => f.write_str("Riot ID is missing the game name"),
            Self::MissingTagLine => f.write_str("Riot ID is missing the tag line"),
            Self::TooManySeparators => f.write_str("Riot ID contains too many # separators"),
        }
    }
}

impl Error for ParseRiotIdError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_riot_id() {
        let riot_id = RiotId::parse("Hide on bush#KR1").unwrap();

        assert_eq!(riot_id.game_name(), "Hide on bush");
        assert_eq!(riot_id.tag_line(), "KR1");
        assert_eq!(riot_id.to_string(), "Hide on bush#KR1");
    }

    #[test]
    fn trims_outer_and_inner_spaces() {
        let riot_id = RiotId::parse("  Player Name  #  EUW  ").unwrap();

        assert_eq!(riot_id.game_name(), "Player Name");
        assert_eq!(riot_id.tag_line(), "EUW");
    }

    #[test]
    fn rejects_empty_input() {
        assert_eq!(RiotId::parse("  "), Err(ParseRiotIdError::Empty));
    }

    #[test]
    fn rejects_missing_separator() {
        assert_eq!(
            RiotId::parse("PlayerName"),
            Err(ParseRiotIdError::MissingSeparator)
        );
    }

    #[test]
    fn rejects_missing_game_name() {
        assert_eq!(
            RiotId::parse("#EUW"),
            Err(ParseRiotIdError::MissingGameName)
        );
    }

    #[test]
    fn rejects_missing_tag_line() {
        assert_eq!(
            RiotId::parse("Player#"),
            Err(ParseRiotIdError::MissingTagLine)
        );
    }

    #[test]
    fn rejects_too_many_separators() {
        assert_eq!(
            RiotId::parse("Player#EUW#extra"),
            Err(ParseRiotIdError::TooManySeparators)
        );
    }
}
