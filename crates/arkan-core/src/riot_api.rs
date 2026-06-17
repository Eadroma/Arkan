use std::error::Error;
use std::fmt::{self, Display, Formatter};

use serde::Deserialize;

use crate::{PlatformRoute, RegionalRoute, RiotId};

#[derive(Debug, Clone)]
pub struct RiotApiClient {
    api_key: String,
    http: reqwest::Client,
}

impl RiotApiClient {
    pub fn new(api_key: impl Into<String>) -> Result<Self, RiotApiError> {
        Self::with_http_client(api_key, reqwest::Client::new())
    }

    pub fn with_http_client(
        api_key: impl Into<String>,
        http: reqwest::Client,
    ) -> Result<Self, RiotApiError> {
        let api_key = api_key.into().trim().to_owned();

        if api_key.is_empty() {
            return Err(RiotApiError::MissingApiKey);
        }

        Ok(Self { api_key, http })
    }

    pub async fn account_by_riot_id(
        &self,
        route: RegionalRoute,
        riot_id: &RiotId,
    ) -> Result<RiotAccount, RiotApiError> {
        let url = account_by_riot_id_url(route, riot_id);
        let response = self
            .http
            .get(url)
            .header("X-Riot-Token", &self.api_key)
            .send()
            .await
            .map_err(RiotApiError::Http)?;

        let status = response.status();

        if !status.is_success() {
            return Err(RiotApiError::RiotStatus(status.as_u16()));
        }

        response
            .json::<RiotAccount>()
            .await
            .map_err(RiotApiError::Http)
    }

    pub async fn summoner_by_puuid(
        &self,
        route: PlatformRoute,
        puuid: &str,
    ) -> Result<RiotSummoner, RiotApiError> {
        let url = summoner_by_puuid_url(route, puuid);
        let response = self
            .http
            .get(url)
            .header("X-Riot-Token", &self.api_key)
            .send()
            .await
            .map_err(RiotApiError::Http)?;

        let status = response.status();

        if !status.is_success() {
            return Err(RiotApiError::RiotStatus(status.as_u16()));
        }

        response
            .json::<RiotSummoner>()
            .await
            .map_err(RiotApiError::Http)
    }
}

pub fn account_by_riot_id_url(route: RegionalRoute, riot_id: &RiotId) -> String {
    format!(
        "https://{}/riot/account/v1/accounts/by-riot-id/{}/{}",
        route.host(),
        encode_path_segment(riot_id.game_name()),
        encode_path_segment(riot_id.tag_line())
    )
}

pub fn summoner_by_puuid_url(route: PlatformRoute, puuid: &str) -> String {
    format!(
        "https://{}/lol/summoner/v4/summoners/by-puuid/{}",
        route.host(),
        encode_path_segment(puuid)
    )
}

fn encode_path_segment(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            _ => {
                let hex = format!("%{byte:02X}");
                hex.chars().collect::<Vec<_>>()
            }
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiotAccount {
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiotSummoner {
    pub id: Option<String>,
    pub account_id: Option<String>,
    pub puuid: String,
    pub profile_icon_id: u32,
    pub revision_date: Option<i64>,
    pub summoner_level: u32,
}

#[derive(Debug)]
pub enum RiotApiError {
    MissingApiKey,
    Http(reqwest::Error),
    RiotStatus(u16),
}

impl Display for RiotApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingApiKey => f.write_str("Riot API key is missing"),
            Self::Http(error) => write!(f, "Riot API HTTP error: {error}"),
            Self::RiotStatus(status) => write!(f, "Riot API returned HTTP {status}"),
        }
    }
}

impl Error for RiotApiError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Http(error) => Some(error),
            Self::MissingApiKey | Self::RiotStatus(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RegionalRoute;

    #[test]
    fn rejects_empty_api_key() {
        let error = RiotApiClient::new("  ").unwrap_err();

        assert!(matches!(error, RiotApiError::MissingApiKey));
    }

    #[test]
    fn builds_account_by_riot_id_url_with_encoding() {
        let riot_id = RiotId::parse("Hide on bush#KR1").unwrap();

        assert_eq!(
            account_by_riot_id_url(RegionalRoute::Asia, &riot_id),
            "https://asia.api.riotgames.com/riot/account/v1/accounts/by-riot-id/Hide%20on%20bush/KR1"
        );
    }

    #[test]
    fn builds_account_by_riot_id_url_for_europe() {
        let riot_id = RiotId::parse("PrincesseMargaux#9096").unwrap();

        assert_eq!(
            account_by_riot_id_url(RegionalRoute::Europe, &riot_id),
            "https://europe.api.riotgames.com/riot/account/v1/accounts/by-riot-id/PrincesseMargaux/9096"
        );
    }

    #[test]
    fn builds_summoner_by_puuid_url_for_platform() {
        assert_eq!(
            summoner_by_puuid_url(PlatformRoute::Euw1, "puuid value"),
            "https://euw1.api.riotgames.com/lol/summoner/v4/summoners/by-puuid/puuid%20value"
        );
    }

    #[test]
    fn deserializes_riot_account_response() {
        let json = r#"{
            "puuid": "abc",
            "gameName": "PrincesseMargaux",
            "tagLine": "9096"
        }"#;

        let account = serde_json::from_str::<RiotAccount>(json).unwrap();

        assert_eq!(
            account,
            RiotAccount {
                puuid: "abc".to_owned(),
                game_name: "PrincesseMargaux".to_owned(),
                tag_line: "9096".to_owned(),
            }
        );
    }

    #[test]
    fn deserializes_riot_summoner_response() {
        let json = r#"{
            "id": "summoner-id",
            "accountId": "account-id",
            "puuid": "abc",
            "profileIconId": 588,
            "summonerLevel": 175
        }"#;

        let summoner = serde_json::from_str::<RiotSummoner>(json).unwrap();

        assert_eq!(
            summoner,
            RiotSummoner {
                id: Some("summoner-id".to_owned()),
                account_id: Some("account-id".to_owned()),
                puuid: "abc".to_owned(),
                profile_icon_id: 588,
                revision_date: None,
                summoner_level: 175,
            }
        );
    }

    #[test]
    fn deserializes_minimal_riot_summoner_response() {
        let json = r#"{
            "puuid": "abc",
            "profileIconId": 588,
            "summonerLevel": 175
        }"#;

        let summoner = serde_json::from_str::<RiotSummoner>(json).unwrap();

        assert_eq!(summoner.puuid, "abc");
        assert_eq!(summoner.profile_icon_id, 588);
        assert_eq!(summoner.summoner_level, 175);
    }
}
