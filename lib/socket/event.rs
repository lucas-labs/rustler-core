use {
    serde::{Deserialize, Serialize},
    serde_json::{Map, Value},
};

pub type Data = Map<String, Value>;

#[derive(Serialize, Deserialize)]
pub struct WsEvent {
    pub event: String,
    pub data: Data,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    #[serde(rename = "errorCode")]
    pub error_code: u16,
    pub msg: String,
}
