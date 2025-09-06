use std::collections::HashMap;
use std::sync::RwLock;

use log::info;

use crate::problem::Problem;

use icfpc2025::problem_definition::ProblemDefinitions;
use icfpc2025::types::{self, *};
pub use types::{ExploreRequest, GuessRequest, SelectRequest};

pub struct Server {
    definitions: ProblemDefinitions,
    problems: RwLock<HashMap<String, Problem>>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            definitions: ProblemDefinitions::new(),
            problems: RwLock::new(HashMap::new()),
        }
    }

    pub fn select(&self, req: SelectRequest) -> Result<SelectResponse, String> {
        let definition = self
            .definitions
            .get(&req.problem_name)
            .ok_or("Problem definition not found")?;

        let problem = Problem::new(definition.size);
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

    fn guess_impl(problem: &Problem, req: GuessRequest) -> Result<GuessResponse, String> {
        let starting_room = req.map.starting_room;

        let mut connections = vec![vec![usize::MAX; 6]; req.map.rooms.len()];
        for connection in req.map.connections.iter() {
            connections[connection.from.room][connection.from.door] = connection.to.room;
            connections[connection.to.room][connection.to.door] = connection.from.room;
        }

        let rooms = req.map.rooms;

        let mut print_connections = format!("Starting room: {}", starting_room);
        for (i, connection) in connections.iter().enumerate() {
            print_connections += &format!("\n{}.{}:", i, rooms[i]);
            for c in connection.iter() {
                print_connections += &format!(" {}", c);
            }
        }
        info!("Attempting to guess:\n{}", print_connections);

        let correct = problem.guess(rooms, starting_room, connections)?;

        return Ok(GuessResponse { correct });
    }

    pub fn guess(&self, req: GuessRequest, keep_problem: bool) -> Result<GuessResponse, String> {
        if !keep_problem {
            let problem = self
                .problems
                .write()
                .unwrap()
                .remove(&req.id)
                .ok_or("Problem not found")?;
            Self::guess_impl(&problem, req)
        } else {
            let problems = self.problems.read().unwrap();
            let problem = problems.get(&req.id).ok_or("Problem not found")?;
            Self::guess_impl(problem, req)
        }
    }
}
