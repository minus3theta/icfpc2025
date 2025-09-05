use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::RwLock;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use log::info;

use crate::problem::Problem;

pub struct Server {
    problems: RwLock<HashMap<String, Problem>>,
}

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

#[derive(Deserialize)]
pub struct ExploreRequest {
    id: String,
    plans: Vec<String>,
}

#[derive(Serialize)]
pub struct ExploreResponse {
    results: Vec<Vec<i8>>,
    #[serde(rename = "queryCount")]
    query_count: usize,
}

#[derive(Deserialize)]
pub struct GuessRequest {
    id: String,
    map: GuessReuqestMap,
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
    room: usize,
    door: usize,
}

#[derive(Serialize)]
pub struct GuessResponse {
    correct: bool,
}

impl Server {
    pub fn new() -> Self {
        Self {
            problems: RwLock::new(HashMap::new()),
        }
    }

    pub fn select(&self, req: SelectRequest) -> Result<SelectResponse, String> {
        let problem = Problem::new(3);
        info!("Problem generated:\n{}", problem.pretty_print());

        self.problems
            .write()
            .unwrap()
            .insert(req.id.clone(), problem);

        Ok(SelectResponse {
            problem_name: req.problem_name.clone(),
        })
    }

    pub fn explore(&self, req: ExploreRequest) -> Result<ExploreResponse, String> {
        let problems = self.problems.read().unwrap();
        let problem = problems.get(&req.id).ok_or("Problem not found")?;

        let mut results = Vec::<Vec<i8>>::new();
        for plan in req.plans {
            results.push(
                problem
                    .explore(&plan)?
                    .into_iter()
                    .map(|v| (v % 4) as i8)
                    .collect(),
            );
        }
        let query_count = results.len();
        Ok(ExploreResponse {
            results,
            query_count,
        })
    }

    pub fn guess(&self, req: GuessRequest) -> Result<GuessResponse, String> {
        let problems = self.problems.read().unwrap();
        let problem = problems.get(&req.id).ok_or("Problem not found")?;

        let mut positions = Vec::<HashSet<usize>>::new();
        for _ in 0..4 {
            positions.push(HashSet::new());
        }
        for (i, room) in req.map.rooms.iter().enumerate() {
            positions[*room as usize].insert(i);
        }

        // 2 bit の room から実際の room の割り当てを列挙する
        for perm in positions
            .iter()
            .map(|p| p.iter().permutations(p.len()))
            .multi_cartesian_product()
        {
            // 2 bit の room から実際の room の割り当てを作成する
            let mut mapping = vec![usize::MAX; req.map.rooms.len()];
            for (i, p) in perm.iter().enumerate() {
                for (j, q) in p.iter().enumerate() {
                    mapping[**q] = i + j * 4;
                }
            }

            let starting_room = mapping[req.map.starting_room];

            let mut connections = vec![vec![usize::MAX; 6]; req.map.rooms.len()];
            for connection in req.map.connections.iter() {
                connections[mapping[connection.from.room]][connection.from.door] =
                    mapping[connection.to.room];
                connections[mapping[connection.to.room]][connection.to.door] =
                    mapping[connection.from.room];
            }

            let mut print_connections = format!("Starting room: {}", starting_room);
            for i in 0..connections.len() {
                print_connections += &format!("\n{}:", i);
                for j in 0..connections[i].len() {
                    print_connections += &format!(" {}", connections[i][j]);
                }
            }
            info!("Attempting to guess:\n{}", print_connections);

            if problem.guess(starting_room, connections)? {
                return Ok(GuessResponse { correct: true });
            }
        }

        Ok(GuessResponse { correct: false })
    }
}
