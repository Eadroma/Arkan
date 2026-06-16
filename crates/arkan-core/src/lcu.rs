use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeagueClientLockfile {
    process_name: String,
    process_id: u32,
    port: u16,
    password: String,
    protocol: LeagueClientProtocol,
}

impl LeagueClientLockfile {
    pub fn parse(input: &str) -> Result<Self, ParseLockfileError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Err(ParseLockfileError::Empty);
        }

        let parts = trimmed.split(':').collect::<Vec<_>>();

        if parts.len() != 5 {
            return Err(ParseLockfileError::InvalidFieldCount {
                actual: parts.len(),
            });
        }

        let process_name = parts[0].trim();
        let process_id = parts[1]
            .trim()
            .parse::<u32>()
            .map_err(|_| ParseLockfileError::InvalidProcessId)?;
        let port = parts[2]
            .trim()
            .parse::<u16>()
            .map_err(|_| ParseLockfileError::InvalidPort)?;
        let password = parts[3].trim();
        let protocol = parts[4].trim().parse::<LeagueClientProtocol>()?;

        if process_name.is_empty() {
            return Err(ParseLockfileError::MissingProcessName);
        }

        if password.is_empty() {
            return Err(ParseLockfileError::MissingPassword);
        }

        Ok(Self {
            process_name: process_name.to_owned(),
            process_id,
            port,
            password: password.to_owned(),
            protocol,
        })
    }

    pub fn process_name(&self) -> &str {
        &self.process_name
    }

    pub fn process_id(&self) -> u32 {
        self.process_id
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn protocol(&self) -> LeagueClientProtocol {
        self.protocol
    }

    pub fn base_url(&self) -> String {
        format!("{}://127.0.0.1:{}", self.protocol, self.port)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeagueClientProtocol {
    Http,
    Https,
}

impl Display for LeagueClientProtocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Http => "http",
            Self::Https => "https",
        })
    }
}

impl std::str::FromStr for LeagueClientProtocol {
    type Err = ParseLockfileError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_ascii_lowercase().as_str() {
            "http" => Ok(Self::Http),
            "https" => Ok(Self::Https),
            _ => Err(ParseLockfileError::InvalidProtocol),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseLockfileError {
    Empty,
    InvalidFieldCount { actual: usize },
    MissingProcessName,
    InvalidProcessId,
    InvalidPort,
    MissingPassword,
    InvalidProtocol,
}

impl Display for ParseLockfileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("League Client lockfile is empty"),
            Self::InvalidFieldCount { actual } => {
                write!(f, "League Client lockfile has {actual} fields instead of 5")
            }
            Self::MissingProcessName => {
                f.write_str("League Client lockfile is missing process name")
            }
            Self::InvalidProcessId => f.write_str("League Client lockfile process id is invalid"),
            Self::InvalidPort => f.write_str("League Client lockfile port is invalid"),
            Self::MissingPassword => f.write_str("League Client lockfile is missing password"),
            Self::InvalidProtocol => f.write_str("League Client lockfile protocol is invalid"),
        }
    }
}

impl Error for ParseLockfileError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_lockfile() {
        let lockfile = LeagueClientLockfile::parse("LeagueClient:1234:65500:secret:https").unwrap();

        assert_eq!(lockfile.process_name(), "LeagueClient");
        assert_eq!(lockfile.process_id(), 1234);
        assert_eq!(lockfile.port(), 65500);
        assert_eq!(lockfile.password(), "secret");
        assert_eq!(lockfile.protocol(), LeagueClientProtocol::Https);
        assert_eq!(lockfile.base_url(), "https://127.0.0.1:65500");
    }

    #[test]
    fn rejects_invalid_field_count() {
        assert_eq!(
            LeagueClientLockfile::parse("LeagueClient:1234"),
            Err(ParseLockfileError::InvalidFieldCount { actual: 2 })
        );
    }

    #[test]
    fn rejects_invalid_port() {
        assert_eq!(
            LeagueClientLockfile::parse("LeagueClient:1234:not-a-port:secret:https"),
            Err(ParseLockfileError::InvalidPort)
        );
    }

    #[test]
    fn rejects_invalid_protocol() {
        assert_eq!(
            LeagueClientLockfile::parse("LeagueClient:1234:65500:secret:ftp"),
            Err(ParseLockfileError::InvalidProtocol)
        );
    }
}
