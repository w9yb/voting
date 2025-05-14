use std::collections::{BTreeMap, BTreeSet};

use serde::de::Error as _;

#[derive(Debug)]
pub struct Ballot {
    pub callsign: String,
    pub ranking: Vec<String>,
}

impl<'de> serde::Deserialize<'de> for Ballot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Ballot;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a ballot")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut callsign = None;
                let mut ranking: BTreeMap<u8, _> = BTreeMap::new();
                while let Some((key, mut value)) = map.next_entry::<String, String>()? {
                    if !value.is_empty() {
                        value.make_ascii_uppercase();

                        if !value.is_ascii()
                            || value.len() < 4
                            || value.len() > 6
                            || !value.chars().all(|c| matches!(c, 'A'..='Z' | '0'..='9'))
                        {
                            return Err(A::Error::custom("invalid callsign"));
                        }

                        if key == "callsign" {
                            if callsign.replace(value).is_some() {
                                return Err(A::Error::duplicate_field("callsign"));
                            }
                        } else if let Some(rank) = key.strip_prefix("rank") {
                            ranking.insert(
                                rank.parse().map_err(|_| A::Error::custom("invalid key"))?,
                                value,
                            );
                        }
                    }
                }

                let mut voted_candidates = BTreeSet::new();
                for v in ranking.values() {
                    if !voted_candidates.insert(v) {
                        return Err(A::Error::custom("multiple votes for same candidate"));
                    }
                }

                Ok(Ballot {
                    callsign: callsign.ok_or_else(|| A::Error::missing_field("callsign"))?,
                    ranking: ranking.into_values().collect(),
                })
            }
        }

        deserializer.deserialize_map(Visitor)
    }
}
