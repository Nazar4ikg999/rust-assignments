use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Request {
    
    #[serde(rename = "type")]
    pub r#type: String,
    pub stream: Stream,
    pub gifts: Vec<Gift>,
    pub debug: DebugInfo,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Stream {
    pub user_id: String,
    pub is_private: bool,
    pub settings: u64,
    pub shard_url: String,
    pub public_tariff: PublicTariff,
    pub private_tariff: PrivateTariff,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PublicTariff {
    pub id: u64,
    pub price: u64,
    pub duration: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PrivateTariff {
    pub client_price: u64,
    pub duration: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Gift {
    pub id: u64,
    pub price: u64,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DebugInfo {
    pub duration: String,
    pub at: String,
}
