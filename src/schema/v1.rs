use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Start {
    pub time: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Finish {
    pub time: u64,
    pub ok: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Report {
    pub artifacts: String,
    pub message: String,
    pub r#ref: String,
    pub start: Start,
    pub finish: Option<Finish>,
}

