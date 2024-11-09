use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Gender {
    #[default]
    #[serde(alias = "u")]
    Unspecified,
    #[serde(alias = "m")]
    Male,
    #[serde(alias = "f")]
    Female,
}
