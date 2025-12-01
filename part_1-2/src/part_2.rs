use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Request {
    pub id: u64,
    pub method: String,
    pub endpoint: String,

    pub headers: Headers,
    pub payload: Payload,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Headers {
    pub accept: String,
    pub authorization: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Payload {
    pub active: bool,
    pub role: String,
    pub limit: u32,
}