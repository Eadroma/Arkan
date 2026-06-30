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

    pub async fn summoner_by_id(
        &self,
        route: PlatformRoute,
        encrypted_summoner_id: &str,
    ) -> Result<RiotSummoner, RiotApiError> {
        let url = summoner_by_id_url(route, encrypted_summoner_id);
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

    pub async fn top_league_entries(
        &self,
        route: PlatformRoute,
        tier: RiotTopLeagueTier,
        queue: &str,
    ) -> Result<Vec<RiotLeagueEntry>, RiotApiError> {
        let url = top_league_url(route, tier, queue);
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
            .json::<RiotLeagueList>()
            .await
            .map(|league| league.entries)
            .map_err(RiotApiError::Http)
    }

    pub async fn champion_mastery_top(
        &self,
        route: PlatformRoute,
        puuid: &str,
        count: u8,
    ) -> Result<Vec<RiotChampionMastery>, RiotApiError> {
        let url = champion_mastery_top_url(route, puuid, count);
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
            .json::<Vec<RiotChampionMastery>>()
            .await
            .map_err(RiotApiError::Http)
    }

    pub async fn match_ids_by_puuid(
        &self,
        route: RegionalRoute,
        puuid: &str,
        start: u32,
        count: u8,
    ) -> Result<Vec<String>, RiotApiError> {
        let url = match_ids_by_puuid_url(route, puuid, start, count);
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
            .json::<Vec<String>>()
            .await
            .map_err(RiotApiError::Http)
    }

    pub async fn match_by_id(
        &self,
        route: RegionalRoute,
        match_id: &str,
    ) -> Result<serde_json::Value, RiotApiError> {
        let url = match_by_id_url(route, match_id);
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
            .json::<serde_json::Value>()
            .await
            .map_err(RiotApiError::Http)
    }

    pub async fn match_timeline_by_id(
        &self,
        route: RegionalRoute,
        match_id: &str,
    ) -> Result<serde_json::Value, RiotApiError> {
        let url = match_timeline_by_id_url(route, match_id);
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
            .json::<serde_json::Value>()
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

pub fn summoner_by_id_url(route: PlatformRoute, encrypted_summoner_id: &str) -> String {
    format!(
        "https://{}/lol/summoner/v4/summoners/{}",
        route.host(),
        encode_path_segment(encrypted_summoner_id)
    )
}

pub fn top_league_url(route: PlatformRoute, tier: RiotTopLeagueTier, queue: &str) -> String {
    format!(
        "https://{}/lol/league/v4/{}/by-queue/{}",
        route.host(),
        tier.path_segment(),
        encode_path_segment(queue)
    )
}

pub fn champion_mastery_top_url(route: PlatformRoute, puuid: &str, count: u8) -> String {
    format!(
        "https://{}/lol/champion-mastery/v4/champion-masteries/by-puuid/{}/top?count={}",
        route.host(),
        encode_path_segment(puuid),
        count
    )
}

pub fn match_ids_by_puuid_url(route: RegionalRoute, puuid: &str, start: u32, count: u8) -> String {
    format!(
        "https://{}/lol/match/v5/matches/by-puuid/{}/ids?start={}&count={}",
        route.host(),
        encode_path_segment(puuid),
        start,
        count
    )
}

pub fn match_by_id_url(route: RegionalRoute, match_id: &str) -> String {
    format!(
        "https://{}/lol/match/v5/matches/{}",
        route.host(),
        encode_path_segment(match_id)
    )
}

pub fn match_timeline_by_id_url(route: RegionalRoute, match_id: &str) -> String {
    format!(
        "https://{}/lol/match/v5/matches/{}/timeline",
        route.host(),
        encode_path_segment(match_id)
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiotChampionMastery {
    pub champion_id: u32,
    pub champion_level: u32,
    pub champion_points: u32,
    pub last_play_time: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiotTopLeagueTier {
    Challenger,
    Grandmaster,
    Master,
}

impl RiotTopLeagueTier {
    pub fn path_segment(self) -> &'static str {
        match self {
            Self::Challenger => "challengerleagues",
            Self::Grandmaster => "grandmasterleagues",
            Self::Master => "masterleagues",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiotLeagueList {
    pub tier: String,
    pub league_id: String,
    pub queue: String,
    pub name: String,
    pub entries: Vec<RiotLeagueEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiotLeagueEntry {
    pub summoner_id: String,
    pub league_points: u32,
    pub rank: String,
    pub wins: u32,
    pub losses: u32,
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
    fn builds_summoner_by_id_url_for_platform() {
        assert_eq!(
            summoner_by_id_url(PlatformRoute::Euw1, "encrypted summoner"),
            "https://euw1.api.riotgames.com/lol/summoner/v4/summoners/encrypted%20summoner"
        );
    }

    #[test]
    fn builds_top_league_urls_for_platform() {
        assert_eq!(
            top_league_url(
                PlatformRoute::Euw1,
                RiotTopLeagueTier::Challenger,
                "RANKED_SOLO_5x5"
            ),
            "https://euw1.api.riotgames.com/lol/league/v4/challengerleagues/by-queue/RANKED_SOLO_5x5"
        );
        assert_eq!(
            top_league_url(
                PlatformRoute::Euw1,
                RiotTopLeagueTier::Grandmaster,
                "RANKED_FLEX_SR"
            ),
            "https://euw1.api.riotgames.com/lol/league/v4/grandmasterleagues/by-queue/RANKED_FLEX_SR"
        );
    }

    #[test]
    fn builds_champion_mastery_top_url_for_platform() {
        assert_eq!(
            champion_mastery_top_url(PlatformRoute::Euw1, "puuid value", 5),
            "https://euw1.api.riotgames.com/lol/champion-mastery/v4/champion-masteries/by-puuid/puuid%20value/top?count=5"
        );
    }

    #[test]
    fn builds_match_ids_by_puuid_url_for_regional_route() {
        assert_eq!(
            match_ids_by_puuid_url(RegionalRoute::Europe, "puuid value", 20, 10),
            "https://europe.api.riotgames.com/lol/match/v5/matches/by-puuid/puuid%20value/ids?start=20&count=10"
        );
    }

    #[test]
    fn builds_match_detail_urls_for_regional_route() {
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

    #[test]
    fn deserializes_champion_mastery_response() {
        let json = r#"{
            "championId": 266,
            "championLevel": 7,
            "championPoints": 123456
        }"#;

        let mastery = serde_json::from_str::<RiotChampionMastery>(json).unwrap();

        assert_eq!(
            mastery,
            RiotChampionMastery {
                champion_id: 266,
                champion_level: 7,
                champion_points: 123456,
                last_play_time: None,
            }
        );
    }

    #[test]
    fn deserializes_top_league_response() {
        let json = r#"{
            "tier": "CHALLENGER",
            "leagueId": "league-id",
            "queue": "RANKED_SOLO_5x5",
            "name": "Taric's Avengers",
            "entries": [
                {
                    "summonerId": "encrypted-id",
                    "leaguePoints": 1200,
                    "rank": "I",
                    "wins": 220,
                    "losses": 140
                }
            ]
        }"#;

        let league = serde_json::from_str::<RiotLeagueList>(json).unwrap();

        assert_eq!(league.tier, "CHALLENGER");
        assert_eq!(league.entries[0].summoner_id, "encrypted-id");
        assert_eq!(league.entries[0].league_points, 1200);
    }
}
