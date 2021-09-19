use super::{Amount, ClientId};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Record {
    #[serde(rename = "client")]
    pub client_id: ClientId,
    #[serde(rename = "available")]
    pub balance: Amount,
    #[serde(rename = "held")]
    pub held: Amount,
    #[serde(rename = "total")]
    pub total: Amount,
    #[serde(rename = "locked")]
    pub locked: bool,
}
