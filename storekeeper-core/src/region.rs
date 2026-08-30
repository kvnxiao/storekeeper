//! Unified region enum for all supported games.
//!
//! Each game has its own region string format for API calls. This module
//! provides a unified `Region` enum that can be converted to game-specific
//! region strings.

use crate::error::Error;
use crate::error::Result;
use crate::game_id::GameId;
use serde::Deserialize;
use serde::Serialize;
use strum::VariantArray;

/// Unified region enum for all games.
///
/// UID prefix → Region mapping happens at parse time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, VariantArray)]
#[serde(rename_all = "snake_case")]
pub enum Region {
    /// Chinese mainland servers.
    China,
    /// North/South American servers.
    America,
    /// European servers.
    Europe,
    /// Asian servers, which cover Japan for every game that runs one.
    Asia,
    /// Hong Kong, Macau, and Taiwan servers.
    ChinaHmt,
    /// Southeast Asian servers.
    Sea,
}

impl Region {
    /// Returns the serialized name of this region.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::China => "china",
            Self::America => "america",
            Self::Europe => "europe",
            Self::Asia => "asia",
            Self::ChinaHmt => "china_hmt",
            Self::Sea => "sea",
        }
    }

    fn unsupported(self, game: GameId) -> Error {
        Error::UnsupportedRegion { game, region: self }
    }

    /// Returns the Genshin Impact API region string.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnsupportedRegion`] if the game does not support this
    /// region.
    pub fn genshin_region(self) -> Result<&'static str> {
        match self {
            Self::China => Ok("cn_gf01"),
            Self::America => Ok("os_usa"),
            Self::Europe => Ok("os_euro"),
            Self::Asia => Ok("os_asia"),
            Self::ChinaHmt => Ok("os_cht"),
            Self::Sea => Err(self.unsupported(GameId::GenshinImpact)),
        }
    }

    /// Returns the Honkai: Star Rail API region string.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnsupportedRegion`] if the game does not support this
    /// region.
    pub fn hsr_region(self) -> Result<&'static str> {
        match self {
            Self::China => Ok("prod_gf_cn"),
            Self::America => Ok("prod_official_usa"),
            Self::Europe => Ok("prod_official_eur"),
            Self::Asia => Ok("prod_official_asia"),
            Self::ChinaHmt => Ok("prod_official_cht"),
            Self::Sea => Err(self.unsupported(GameId::HonkaiStarRail)),
        }
    }

    /// Returns the Zenless Zone Zero API region string.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnsupportedRegion`] if the game does not support this
    /// region.
    pub fn zzz_region(self) -> Result<&'static str> {
        match self {
            Self::China => Ok("prod_gf_cn"),
            Self::America => Ok("prod_gf_us"),
            Self::Europe => Ok("prod_gf_eu"),
            Self::Asia => Ok("prod_gf_jp"),
            Self::ChinaHmt => Ok("prod_gf_sg"),
            Self::Sea => Err(self.unsupported(GameId::ZenlessZoneZero)),
        }
    }

    /// Returns the Wuthering Waves API region string.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnsupportedRegion`] if the game does not support this
    /// region.
    pub fn wuwa_region(self) -> Result<&'static str> {
        match self {
            Self::America => Ok("America"),
            Self::Europe => Ok("Europe"),
            Self::Asia => Ok("Asia"),
            Self::ChinaHmt => Ok("HMT"),
            Self::Sea => Ok("SEA"),
            Self::China => Err(self.unsupported(GameId::WutheringWaves)),
        }
    }

    /// Parses region from a Genshin Impact UID.
    ///
    /// # Errors
    ///
    /// Returns an error if the UID format is invalid or the region cannot be
    /// determined.
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
            "9" => Ok(Self::ChinaHmt),
            _ => Err(Error::UnknownUidRegion(uid.to_string())),
        }
    }

    /// Parses region from a Honkai: Star Rail UID.
    ///
    /// # Errors
    ///
    /// Returns an error if the UID format is invalid or the region cannot be
    /// determined.
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
            "9" => Ok(Self::ChinaHmt),
            _ => Err(Error::UnknownUidRegion(uid.to_string())),
        }
    }

    /// Parses region from a Zenless Zone Zero UID.
    ///
    /// 8-digit UIDs are Chinese servers, 10-digit UIDs are global with prefix
    /// mapping.
    ///
    /// # Errors
    ///
    /// Returns an error if the UID format is invalid or the region cannot be
    /// determined.
    pub fn from_zzz_uid(uid: &str) -> Result<Self> {
        match uid.len() {
            8 => Ok(Self::China),
            10 => {
                let prefix = uid
                    .get(..2)
                    .ok_or_else(|| Error::UnknownUidRegion(uid.to_string()))?;

                match prefix {
                    "10" => Ok(Self::America),
                    "13" => Ok(Self::Asia),
                    "15" => Ok(Self::Europe),
                    "17" => Ok(Self::ChinaHmt),
                    _ => Err(Error::UnknownUidRegion(uid.to_string())),
                }
            }
            _ => Err(Error::UnknownUidRegion(uid.to_string())),
        }
    }

    /// Parses region from a Wuthering Waves UID.
    ///
    /// The first digit determines the region.
    ///
    /// # Errors
    ///
    /// Returns an error if the UID format is invalid or the region cannot be
    /// determined.
    pub fn from_wuwa_uid(uid: &str) -> Result<Self> {
        let first = uid
            .chars()
            .next()
            .ok_or_else(|| Error::UnknownUidRegion(uid.to_string()))?;

        match first {
            '5' => Ok(Self::America),
            '6' => Ok(Self::Europe),
            '7' => Ok(Self::Asia),
            '8' => Ok(Self::ChinaHmt),
            '9' => Ok(Self::Sea),
            _ => Err(Error::UnknownUidRegion(uid.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Detector = fn(&str) -> Result<Region>;
    type Lookup = fn(Region) -> Result<&'static str>;
    type ServerCase = (
        &'static str,
        Detector,
        Lookup,
        &'static [(&'static str, &'static str)],
    );

    const SUPPORTED_GENSHIN: &[(&str, &str)] = &[
        ("china", "cn_gf01"),
        ("america", "os_usa"),
        ("europe", "os_euro"),
        ("asia", "os_asia"),
        ("china_hmt", "os_cht"),
    ];
    const SUPPORTED_HSR: &[(&str, &str)] = &[
        ("china", "prod_gf_cn"),
        ("america", "prod_official_usa"),
        ("europe", "prod_official_eur"),
        ("asia", "prod_official_asia"),
        ("china_hmt", "prod_official_cht"),
    ];
    const SUPPORTED_ZZZ: &[(&str, &str)] = &[
        ("china", "prod_gf_cn"),
        ("america", "prod_gf_us"),
        ("europe", "prod_gf_eu"),
        ("asia", "prod_gf_jp"),
        ("china_hmt", "prod_gf_sg"),
    ];
    const SUPPORTED_WUWA: &[(&str, &str)] = &[
        ("america", "America"),
        ("europe", "Europe"),
        ("asia", "Asia"),
        ("china_hmt", "HMT"),
        ("sea", "SEA"),
    ];

    fn supported(lookup: fn(Region) -> Result<&'static str>) -> Vec<(&'static str, &'static str)> {
        Region::VARIANTS
            .iter()
            .copied()
            .filter_map(|region| lookup(region).ok().map(|server| (region.as_str(), server)))
            .collect()
    }

    #[test]
    fn genshin_uid_parsing() {
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
    fn zzz_uid_parsing() {
        assert_eq!(
            Region::from_zzz_uid("12345678").expect("valid uid"),
            Region::China
        );
        assert_eq!(
            Region::from_zzz_uid("1012345678").expect("valid uid"),
            Region::America
        );
        assert_eq!(
            Region::from_zzz_uid("1312345678").expect("valid uid"),
            Region::Asia
        );
        assert_eq!(
            Region::from_zzz_uid("1712345678").expect("valid uid"),
            Region::ChinaHmt
        );
    }

    #[test]
    fn wuwa_uid_parsing() {
        assert_eq!(
            Region::from_wuwa_uid("502763418").expect("valid uid"),
            Region::America
        );
        assert_eq!(
            Region::from_wuwa_uid("600000001").expect("valid uid"),
            Region::Europe
        );
    }

    #[test]
    fn china_hmt_serializes_under_its_own_name() {
        let json = serde_json::to_value(Region::ChinaHmt).expect("should serialize");
        assert_eq!(json, serde_json::json!("china_hmt"));
    }

    #[test]
    fn every_region_either_maps_to_its_own_server_or_errors() {
        type Lookup = (&'static str, fn(Region) -> Result<&'static str>);
        let games: [Lookup; 4] = [
            ("genshin", Region::genshin_region),
            ("hsr", Region::hsr_region),
            ("zzz", Region::zzz_region),
            ("wuwa", Region::wuwa_region),
        ];

        for (game, lookup) in games {
            let mut seen: Vec<(&str, &str)> = Vec::new();

            for region in Region::VARIANTS {
                let Ok(server) = lookup(*region) else {
                    continue;
                };

                assert!(
                    !seen.iter().any(|(_, s)| *s == server),
                    "{game} maps {} onto `{server}`, already used by {:?}",
                    region.as_str(),
                    seen.iter().find(|(_, s)| *s == server).map(|(r, _)| r)
                );
                seen.push((region.as_str(), server));
            }
        }
    }

    #[test]
    fn an_unsupported_region_names_the_game_and_the_region() {
        let err = Region::Sea
            .genshin_region()
            .expect_err("Genshin runs no Southeast Asian server");

        assert_eq!(err.to_string(), "Genshin Impact has no sea server");
    }

    #[test]
    fn each_game_serves_exactly_its_own_servers() {
        for (game, lookup, expected) in [
            (
                "genshin",
                Region::genshin_region as fn(Region) -> _,
                SUPPORTED_GENSHIN,
            ),
            ("hsr", Region::hsr_region, SUPPORTED_HSR),
            ("zzz", Region::zzz_region, SUPPORTED_ZZZ),
            ("wuwa", Region::wuwa_region, SUPPORTED_WUWA),
        ] {
            assert_eq!(
                supported(lookup),
                expected,
                "{game} serves the wrong servers"
            );
        }
    }

    #[test]
    fn as_str_agrees_with_serde_for_every_variant() {
        for region in Region::VARIANTS {
            let serialized = serde_json::to_value(region).expect("region should serialize");
            assert_eq!(serialized, serde_json::json!(region.as_str()));
        }
    }

    #[test]
    fn every_game_serves_exactly_what_its_uid_rule_detects() {
        let games: [(&str, Detector, Lookup); 4] = [
            ("genshin", Region::from_genshin_uid, Region::genshin_region),
            ("hsr", Region::from_hsr_uid, Region::hsr_region),
            ("zzz", Region::from_zzz_uid, Region::zzz_region),
            ("wuwa", Region::from_wuwa_uid, Region::wuwa_region),
        ];

        for (game, detect, lookup) in games {
            let mut detected: Vec<&str> = candidate_uids()
                .iter()
                .filter_map(|uid| detect(uid).ok())
                .map(Region::as_str)
                .collect();
            detected.sort_unstable();
            detected.dedup();

            let mut served: Vec<&str> = Region::VARIANTS
                .iter()
                .copied()
                .filter(|region| lookup(*region).is_ok())
                .map(Region::as_str)
                .collect();
            served.sort_unstable();

            assert_eq!(
                detected, served,
                "{game} serves a different set than its UID rule detects"
            );
        }
    }

    /// Generate UID candidates for every one- and two-digit prefix at lengths
    /// 8 through 10.
    fn candidate_uids() -> Vec<String> {
        let mut uids = Vec::new();
        for prefix in 0..100u32 {
            for total in [8, 9, 10] {
                let prefix = prefix.to_string();
                if prefix.len() < total {
                    uids.push(format!("{prefix}{}", "0".repeat(total - prefix.len())));
                }
            }
        }
        uids
    }

    /// The API string a UID resolves to is the wire contract; relabelling a
    /// region must leave every one of these untouched.
    #[test]
    fn each_uid_resolves_to_a_fixed_api_server() {
        let cases: [ServerCase; 4] = [
            (
                "genshin",
                Region::from_genshin_uid,
                Region::genshin_region,
                &[
                    ("100000001", "cn_gf01"),
                    ("600000001", "os_usa"),
                    ("700000001", "os_euro"),
                    ("800000001", "os_asia"),
                    ("900000001", "os_cht"),
                ],
            ),
            (
                "hsr",
                Region::from_hsr_uid,
                Region::hsr_region,
                &[
                    ("100000001", "prod_gf_cn"),
                    ("600000001", "prod_official_usa"),
                    ("700000001", "prod_official_eur"),
                    ("800000001", "prod_official_asia"),
                    ("900000001", "prod_official_cht"),
                ],
            ),
            (
                "zzz",
                Region::from_zzz_uid,
                Region::zzz_region,
                &[
                    ("12345678", "prod_gf_cn"),
                    ("1012345678", "prod_gf_us"),
                    ("1312345678", "prod_gf_jp"),
                    ("1512345678", "prod_gf_eu"),
                    ("1712345678", "prod_gf_sg"),
                ],
            ),
            (
                "wuwa",
                Region::from_wuwa_uid,
                Region::wuwa_region,
                &[
                    ("500000001", "America"),
                    ("600000001", "Europe"),
                    ("700000001", "Asia"),
                    ("800000001", "HMT"),
                    ("900000001", "SEA"),
                ],
            ),
        ];

        for (game, detect, lookup, expected) in cases {
            for (uid, server) in expected {
                let resolved = detect(uid).and_then(lookup);
                assert_eq!(
                    resolved.ok(),
                    Some(*server),
                    "{game} should send {server} for the uid {uid}"
                );
            }
        }
    }
}
