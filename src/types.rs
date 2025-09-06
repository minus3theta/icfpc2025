use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SelectRequest {
    pub id: String,
    #[serde(rename = "problemName")]
    pub problem_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SelectResponse {
    #[serde(rename = "problemName")]
    pub problem_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExploreRequest {
    pub id: String,
    pub plans: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExploreResponse {
    pub results: Vec<Vec<i8>>,
    #[serde(rename = "queryCount")]
    pub query_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuessRequest {
    pub id: String,
    pub map: GuessRequestMap,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuessRequestMap {
    pub rooms: Vec<i8>,
    #[serde(rename = "startingRoom")]
    pub starting_room: usize,
    pub connections: Vec<GuessRequestConnection>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuessRequestConnection {
    pub from: GuessRequestRoom,
    pub to: GuessRequestRoom,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuessRequestRoom {
    pub room: usize,
    pub door: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuessResponse {
    pub correct: bool,
}
