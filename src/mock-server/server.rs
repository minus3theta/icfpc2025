use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::RwLock;

use itertools::Itertools;

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
            for (i, connection) in connections.iter().enumerate() {
                print_connections += &format!("\n{}:", i);
                for c in connection.iter() {
                    print_connections += &format!(" {}", c);
                }
            }
            info!("Attempting to guess:\n{}", print_connections);

            if problem.guess(starting_room, connections)? {
                return Ok(GuessResponse { correct: true });
            }
        }

        Ok(GuessResponse { correct: false })
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
