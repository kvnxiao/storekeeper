//! Unified region enum for all supported games.
//!
//! Each game has its own region string format for API calls. This module
//! provides a unified `Region` enum that can be converted to game-specific
//! region strings.

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::error::{Error, Result};

/// Unified region enum for all games.
///
/// UID prefix → Region mapping happens at parse time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Region {
    /// Chinese mainland servers.
    #[strum(serialize = "china")]
    China,
    /// North/South American servers.
    #[strum(serialize = "america")]
    America,
    /// European servers.
    #[strum(serialize = "europe")]
    Europe,
    /// Asian servers (excluding Japan for some games).
    #[strum(serialize = "asia")]
    Asia,
    /// Chinese Traditional (Taiwan/HK/Macau).
    #[strum(serialize = "cht")]
    Cht,
    /// Japanese servers.
    #[strum(serialize = "japan")]
    Japan,
    /// Southeast Asian servers.
    #[strum(serialize = "sea")]
    Sea,
}

impl Region {
    /// Returns the Genshin Impact API region string.
    #[must_use]
    pub fn genshin_region(self) -> &'static str {
        match self {
            Self::China => "cn_gf01",
            Self::America => "os_usa",
            Self::Europe => "os_euro",
            Self::Asia | Self::Japan | Self::Sea => "os_asia",
            Self::Cht => "os_cht",
        }
    }

    /// Returns the Honkai: Star Rail API region string.
    #[must_use]
    pub fn hsr_region(self) -> &'static str {
        match self {
            Self::China => "prod_gf_cn",
            Self::America => "prod_official_usa",
            Self::Europe => "prod_official_eur",
            Self::Asia | Self::Japan | Self::Sea => "prod_official_asia",
            Self::Cht => "prod_official_cht",
        }
    }

    /// Returns the Zenless Zone Zero API region string.
    #[must_use]
    pub fn zzz_region(self) -> &'static str {
        match self {
            Self::China => "prod_gf_cn",
            Self::America => "prod_gf_us",
            Self::Europe => "prod_gf_eu",
            Self::Asia | Self::Cht | Self::Sea => "prod_gf_sg",
            Self::Japan => "prod_gf_jp",
        }
    }

    /// Returns the Wuthering Waves API region string.
    #[must_use]
    pub fn wuwa_region(self) -> &'static str {
        match self {
            Self::China => "China",
            Self::America => "America",
            Self::Europe => "Europe",
            Self::Asia | Self::Japan => "Asia",
            Self::Cht => "HMT",
            Self::Sea => "SEA",
        }
    }

    /// Parses region from a Genshin Impact UID.
    ///
    /// # Errors
    ///
    /// Returns an error if the UID format is invalid or the region cannot be determined.
    pub fn from_genshin_uid(uid: &str) -> Result<Self> {
        let prefix = uid
            .get(..uid.len().saturating_sub(8))
            .filter(|p| !p.is_empty())
            .ok_or_else(|| Error::UnknownUidRegion(uid.to_string()))?;

        match prefix {
            "1" | "2" | "3" | "5" => Ok(Self::China),
            "6" => Ok(Self::America),
            "7" => Ok(Self::Europe),
            "8" | "18" => Ok(Self::Asia),
            "9" => Ok(Self::Cht),
            _ => Err(Error::UnknownUidRegion(uid.to_string())),
        }
    }

    /// Parses region from a Honkai: Star Rail UID.
    ///
    /// # Errors
    ///
    /// Returns an error if the UID format is invalid or the region cannot be determined.
    pub fn from_hsr_uid(uid: &str) -> Result<Self> {
        let prefix = uid
            .get(..uid.len().saturating_sub(8))
            .filter(|p| !p.is_empty())
            .ok_or_else(|| Error::UnknownUidRegion(uid.to_string()))?;

        match prefix {
            "1" | "2" | "5" => Ok(Self::China),
            "6" => Ok(Self::America),
            "7" => Ok(Self::Europe),
            "8" => Ok(Self::Asia),
            "9" => Ok(Self::Cht),
            _ => Err(Error::UnknownUidRegion(uid.to_string())),
        }
    }

    /// Parses region from a Zenless Zone Zero UID.
    ///
    /// 8-digit UIDs are Chinese servers, 10-digit UIDs are global with prefix mapping.
    ///
    /// # Errors
    ///
    /// Returns an error if the UID format is invalid or the region cannot be determined.
    pub fn from_zzz_uid(uid: &str) -> Result<Self> {
        match uid.len() {
            8 => Ok(Self::China),
            10 => {
                let prefix = uid
                    .get(..2)
                    .ok_or_else(|| Error::UnknownUidRegion(uid.to_string()))?;

                match prefix {
                    "10" => Ok(Self::America),
                    "13" => Ok(Self::Japan),
                    "15" => Ok(Self::Europe),
                    "17" => Ok(Self::Asia),
                    _ => Err(Error::UnknownUidRegion(uid.to_string())),
                }
            }
            _ => Err(Error::UnknownUidRegion(uid.to_string())),
        }
    }

    /// Parses region from a Wuthering Waves player ID.
    ///
    /// The first digit determines the region.
    ///
    /// # Errors
    ///
    /// Returns an error if the player ID format is invalid or the region cannot be determined.
    pub fn from_wuwa_player_id(player_id: &str) -> Result<Self> {
        let first = player_id
            .chars()
            .next()
            .ok_or_else(|| Error::UnknownUidRegion(player_id.to_string()))?;

        match first {
            '5' => Ok(Self::America),
            '6' => Ok(Self::Europe),
            '7' => Ok(Self::Asia),
            '8' => Ok(Self::Cht),
            '9' => Ok(Self::Sea),
            _ => Err(Error::UnknownUidRegion(player_id.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genshin_uid_parsing() {
        assert_eq!(
            Region::from_genshin_uid("700000001").expect("valid uid"),
            Region::Europe
        );
        assert_eq!(
            Region::from_genshin_uid("600000001").expect("valid uid"),
            Region::America
        );
        assert_eq!(
            Region::from_genshin_uid("800000001").expect("valid uid"),
            Region::Asia
        );
    }

    #[test]
    fn test_zzz_uid_parsing() {
        // 8-digit = China
        assert_eq!(
            Region::from_zzz_uid("12345678").expect("valid uid"),
            Region::China
        );
        // 10-digit with prefix
        assert_eq!(
            Region::from_zzz_uid("1012345678").expect("valid uid"),
            Region::America
        );
        assert_eq!(
            Region::from_zzz_uid("1312345678").expect("valid uid"),
            Region::Japan
        );
    }

    #[test]
    fn test_wuwa_player_id_parsing() {
        assert_eq!(
            Region::from_wuwa_player_id("502763418").expect("valid id"),
            Region::America
        );
        assert_eq!(
            Region::from_wuwa_player_id("600000001").expect("valid id"),
            Region::Europe
        );
    }
}
