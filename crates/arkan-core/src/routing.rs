use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformRoute {
    Br1,
    Eun1,
    Euw1,
    Jp1,
    Kr,
    La1,
    La2,
    Na1,
    Oc1,
    Tr1,
    Ru,
    Ph2,
    Sg2,
    Th2,
    Tw2,
    Vn2,
}

impl PlatformRoute {
    pub fn host(self) -> &'static str {
        match self {
            Self::Br1 => "br1.api.riotgames.com",
            Self::Eun1 => "eun1.api.riotgames.com",
            Self::Euw1 => "euw1.api.riotgames.com",
            Self::Jp1 => "jp1.api.riotgames.com",
            Self::Kr => "kr.api.riotgames.com",
            Self::La1 => "la1.api.riotgames.com",
            Self::La2 => "la2.api.riotgames.com",
            Self::Na1 => "na1.api.riotgames.com",
            Self::Oc1 => "oc1.api.riotgames.com",
            Self::Tr1 => "tr1.api.riotgames.com",
            Self::Ru => "ru.api.riotgames.com",
            Self::Ph2 => "ph2.api.riotgames.com",
            Self::Sg2 => "sg2.api.riotgames.com",
            Self::Th2 => "th2.api.riotgames.com",
            Self::Tw2 => "tw2.api.riotgames.com",
            Self::Vn2 => "vn2.api.riotgames.com",
        }
    }

    pub fn regional_route(self) -> RegionalRoute {
        match self {
            Self::Br1 | Self::La1 | Self::La2 | Self::Na1 => RegionalRoute::Americas,
            Self::Eun1 | Self::Euw1 | Self::Ru | Self::Tr1 => RegionalRoute::Europe,
            Self::Jp1 | Self::Kr => RegionalRoute::Asia,
            Self::Oc1 | Self::Ph2 | Self::Sg2 | Self::Th2 | Self::Tw2 | Self::Vn2 => {
                RegionalRoute::Sea
            }
        }
    }
}

impl Display for PlatformRoute {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Br1 => "BR1",
            Self::Eun1 => "EUN1",
            Self::Euw1 => "EUW1",
            Self::Jp1 => "JP1",
            Self::Kr => "KR",
            Self::La1 => "LA1",
            Self::La2 => "LA2",
            Self::Na1 => "NA1",
            Self::Oc1 => "OC1",
            Self::Tr1 => "TR1",
            Self::Ru => "RU",
            Self::Ph2 => "PH2",
            Self::Sg2 => "SG2",
            Self::Th2 => "TH2",
            Self::Tw2 => "TW2",
            Self::Vn2 => "VN2",
        })
    }
}

impl FromStr for PlatformRoute {
    type Err = ParseRouteError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_ascii_uppercase().as_str() {
            "BR1" => Ok(Self::Br1),
            "EUN1" => Ok(Self::Eun1),
            "EUW1" => Ok(Self::Euw1),
            "JP1" => Ok(Self::Jp1),
            "KR" => Ok(Self::Kr),
            "LA1" => Ok(Self::La1),
            "LA2" => Ok(Self::La2),
            "NA1" => Ok(Self::Na1),
            "OC1" => Ok(Self::Oc1),
            "TR1" => Ok(Self::Tr1),
            "RU" => Ok(Self::Ru),
            "PH2" => Ok(Self::Ph2),
            "SG2" => Ok(Self::Sg2),
            "TH2" => Ok(Self::Th2),
            "TW2" => Ok(Self::Tw2),
            "VN2" => Ok(Self::Vn2),
            _ => Err(ParseRouteError),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionalRoute {
    Americas,
    Asia,
    Europe,
    Sea,
}

impl RegionalRoute {
    pub fn host(self) -> &'static str {
        match self {
            Self::Americas => "americas.api.riotgames.com",
            Self::Asia => "asia.api.riotgames.com",
            Self::Europe => "europe.api.riotgames.com",
            Self::Sea => "sea.api.riotgames.com",
        }
    }
}

impl Display for RegionalRoute {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Americas => "AMERICAS",
            Self::Asia => "ASIA",
            Self::Europe => "EUROPE",
            Self::Sea => "SEA",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseRouteError;

impl Display for ParseRouteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("unknown Riot route")
    }
}

impl Error for ParseRouteError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_platform_route_case_insensitively() {
        assert_eq!("euw1".parse::<PlatformRoute>(), Ok(PlatformRoute::Euw1));
        assert_eq!("KR".parse::<PlatformRoute>(), Ok(PlatformRoute::Kr));
    }

    #[test]
    fn maps_platform_to_platform_host() {
        assert_eq!(PlatformRoute::Euw1.host(), "euw1.api.riotgames.com");
        assert_eq!(PlatformRoute::Na1.host(), "na1.api.riotgames.com");
    }

    #[test]
    fn maps_platform_to_regional_route() {
        assert_eq!(PlatformRoute::Euw1.regional_route(), RegionalRoute::Europe);
        assert_eq!(PlatformRoute::Na1.regional_route(), RegionalRoute::Americas);
        assert_eq!(PlatformRoute::Kr.regional_route(), RegionalRoute::Asia);
        assert_eq!(PlatformRoute::Sg2.regional_route(), RegionalRoute::Sea);
    }

    #[test]
    fn maps_regional_route_to_host() {
        assert_eq!(RegionalRoute::Europe.host(), "europe.api.riotgames.com");
    }

    #[test]
    fn rejects_unknown_platform_route() {
        assert!("EU".parse::<PlatformRoute>().is_err());
    }
}
