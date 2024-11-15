use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum Platform {
    Android = 1,
    Ios = 2,
    Web = 3,
    Desktop = 4,
    Mobile = 5,
    Tablet = 6,
    Console = 7,
    SmartTv = 8,
    Other = 9,
}
