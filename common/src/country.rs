use isocountry::CountryCode;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Country(CountryCode);

impl Country {
    pub fn into_code(self) -> String {
        self.0.to_string()
    }
    pub fn into_id(self) -> u32 {
        self.0.numeric_id()
    }
}

impl Default for Country {
    fn default() -> Self {
        Country(CountryCode::USA)
    }
}

impl Serialize for Country {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.to_string().serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for Country {
    fn deserialize<D>(deserializer: D) -> Result<Country, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let s = String::deserialize(deserializer)?;
        let code = match s.len() {
            2 => CountryCode::for_alpha2_caseless(&s),
            3 => CountryCode::for_alpha3_caseless(&s),
            _ => return Err(serde::de::Error::custom("invalid length")),
        };
        match code {
            Ok(code) => Ok(Country(code)),
            Err(err) => Err(serde::de::Error::custom(err)),
        }
    }
}
