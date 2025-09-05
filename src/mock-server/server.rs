use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SelectRequest {
  id: String,
  #[serde(rename = "problemName")]
  problem_name: String,
}

#[derive(Serialize)]
pub struct SelectResponse {
  #[serde(rename = "problemName")]
  problem_name: String,
}

pub async fn select(req: SelectRequest) -> SelectResponse {
  SelectResponse {
    problem_name: req.problem_name.clone(),
  }
}

#[derive(Deserialize)]
pub struct ExploreRequest {
  id: String,
  plans: Vec<String>,
}

#[derive(Serialize)]
pub struct ExploreResponse {
  results: Vec<i8>,
  #[serde(rename = "queryCount")]
  query_count: usize,
}

pub async fn explore(req: ExploreRequest) -> ExploreResponse {
  ExploreResponse {
    results: vec![1; req.plans.len()],
    query_count: req.plans.len(),
  }
}

#[derive(Deserialize)]
pub struct GuessRequest {
  id: String,
  map: String,
}

#[derive(Deserialize)]
struct GuessReuqestMap {
  rooms: Vec<i8>,
  #[serde(rename = "startingRoom")]
  starting_room: usize,
  connections: Vec<GuessRequestConnection>,
}

#[derive(Deserialize)]
struct GuessRequestConnection {
  from: GuessRequestRoom,
  to: GuessRequestRoom,
}

#[derive(Deserialize)]
struct GuessRequestRoom {
  id: usize,
  door: usize,
}

#[derive(Serialize)]
pub struct GuessResponse {
  correct: bool,
}

pub async fn guess(_req: GuessRequest) -> GuessResponse {
  GuessResponse {
    correct: true,
  }
}
