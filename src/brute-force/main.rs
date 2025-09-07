use std::collections::HashMap;
use std::env;

use rand::Rng;

use itertools::Itertools;

use icfpc2025::problem_definition::{ProblemDefinition, ProblemDefinitions};
use icfpc2025::request::*;
use icfpc2025::types::*;

struct BruteForce<R> {
    requester: R,
    definition: ProblemDefinition,
}

impl<R: Requester> BruteForce<R> {
    fn new(problem: String, requester: R) -> Result<Self, Box<dyn std::error::Error>> {
        let select_resp = requester.select(problem.clone(), None)?;
        println!("Selected problem: {:?}", select_resp);

        let definitions = ProblemDefinitions::new();
        let definition = definitions.get(&problem).ok_or("Invalid problem name")?;

        Ok(BruteForce {
            requester,
            definition: definition.clone(),
        })
    }

    fn solve(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let door_count = 6;

        // 乱数の引きによってはかぶることがあるので大きい問題では suffix_count を大きくする
        let suffix_length = self.definition.max_plan_length - 6;
        let suffix_count = 1;

        let mut suffixes = Vec::new();
        let mut rng = rand::rng();
        for _ in 0..suffix_count {
            let mut suffix = String::new();
            for _ in 0..suffix_length {
                suffix.push_str(&rng.random_range(0..door_count).to_string());
            }
            suffixes.push(suffix);
        }
        let prefix = if self.definition.label_rewritable {
            format!("[{}]", rng.random_range(0..4))
        } else {
            "".to_string()
        };
        println!("Prefix: {:?}", prefix);
        let suffixes = suffixes;
        println!("Suffixes: {:?}", suffixes);

        let mut get_ploidy_group = None;

        // suffix 部分の result の digit 置換グループ -> (グループ, suffix 部分の result（部屋ハッシュみたいなもの） -> 部屋番号)
        let mut room_ids = HashMap::<Vec<Vec<i8>>, (usize, HashMap<Vec<Vec<i8>>, usize>)>::new();
        // [部屋に到達できる経路, 部屋番号]
        let mut room_paths = vec![("".to_string(), 0)];
        // 部屋番号 -> 部屋の数字
        let mut room_digits = vec![];

        // 未処理の最初の部屋番号
        let mut unprocessed = 0;

        let mut connections = vec![];

        while unprocessed < room_paths.len() {
            let mut requests = Vec::new();
            if unprocessed == 0 {
                // 最初の部屋だけは特別扱い
                requests.push(None);
            }
            for unprocessed_room in unprocessed..room_paths.len() {
                // 未到達の部屋のすべてのドアを調べる
                for i in 0..door_count {
                    requests.push(Some((unprocessed_room, i)));
                }
            }
            unprocessed = room_paths.len();
            // 最後に suffix を追加して部屋ハッシュのようなものを得られるようにする
            let plans = requests.iter().cartesian_product(suffixes.iter()).map(|(request, suffix)| match request {
                // 最初の部屋だけは特別扱い
                None => prefix.clone(),
                Some((unprocessed_room, door_id)) => prefix.clone() + &room_paths[*unprocessed_room].0.clone() + &door_id.to_string(),
            } + suffix).collect::<Vec<String>>();

            let results = self.explore(&plans)?;

            for (request, chunk) in requests.into_iter().zip_eq(results.chunks(suffixes.len())) {
                let res = chunk.to_vec();
                // 部屋の数字
                let room_digit = res[0][res[0].len() - suffix_length - 1];
                // 部屋ハッシュのようなもの
                let suffix_data = res
                    .iter()
                    .map(|r| r[r.len() - suffix_length - 1..r.len()].to_vec())
                    .collect::<Vec<Vec<i8>>>();
                match request {
                    None => {
                        if self.definition.label_rewritable && room_digit == res[0][0] {
                            return Err("Bad luck!".into());
                        }
                        let room_digit = res[0][0];
                        let rewrite_digit = res[0][1];
                        get_ploidy_group = Some(move |result: &Vec<Vec<i8>>| -> Vec<Vec<i8>> {
                            result
                                .iter()
                                .map(|r| {
                                    r.iter()
                                        .map(|d| if *d == rewrite_digit { room_digit } else { *d })
                                        .collect::<Vec<_>>()
                                })
                                .collect::<Vec<_>>()
                        });
                        // 最初の部屋だけは特別扱い
                        println!(
                            "Found new room {:?}: {:?}.{:?} at {:?}",
                            0,
                            room_digit,
                            suffix_data
                                .iter()
                                .map(|d| d.iter().map(|d| d.to_string()).join(""))
                                .collect::<Vec<String>>(),
                            room_paths.iter().last().unwrap().0
                        );
                        room_ids.insert(
                            get_ploidy_group.unwrap()(&suffix_data),
                            (0, HashMap::from([(suffix_data, 0)])),
                        );
                        for _ in 0..self.definition.ploidy {
                            room_digits.push(room_digit);
                        }
                    }
                    Some((unprocessed_room, door_id)) => {
                        let current_len = room_digits.len();
                        let entry = room_ids
                            .entry(get_ploidy_group.unwrap()(&suffix_data))
                            .or_insert_with(|| {
                                for _ in 0..self.definition.ploidy {
                                    room_digits.push(room_digit);
                                }
                                (current_len, HashMap::new())
                            });
                        let from_room_id = room_paths[unprocessed_room].1;
                        let new_room_id = entry.0 + entry.1.len();
                        let to_room_id = *entry.1.entry(suffix_data.clone()).or_insert_with(|| {
                            let room_path =
                                room_paths[unprocessed_room].0.clone() + &door_id.to_string();
                            // 最後の 1 コピーは補完できるので調査する必要がない
                            if self.definition.ploidy == 1
                                || new_room_id % self.definition.ploidy < self.definition.ploidy - 1
                            {
                                room_paths.push((room_path.clone(), new_room_id));
                            }
                            println!(
                                "Found new room {:?}: {:?}.{:?} at {:?}",
                                new_room_id,
                                room_digit,
                                suffix_data
                                    .iter()
                                    .map(|d| d.iter().map(|d| d.to_string()).join(""))
                                    .collect::<Vec<String>>(),
                                room_path
                            );
                            new_room_id
                        });
                        if to_room_id - entry.0 >= self.definition.ploidy {
                            return Err("Ploidy overflow".into());
                        }
                        let from_room_group =
                            (from_room_id / self.definition.ploidy) * self.definition.ploidy;
                        let to_room_group =
                            (to_room_id / self.definition.ploidy) * self.definition.ploidy;
                        while connections.len() < from_room_group + self.definition.ploidy {
                            connections.push(vec![!0; 6]);
                        }
                        connections[from_room_id][door_id] = to_room_id;
                        println!(
                            "Found connection {}.{}->{}",
                            from_room_id, door_id, to_room_id
                        );

                        // 未調査のドアを補完
                        let mut remaining: usize = (1 << self.definition.ploidy) - 1;
                        for i in 0..self.definition.ploidy {
                            if connections[from_room_group + i][door_id] != !0 {
                                remaining &= !(1
                                    << (connections[from_room_group + i][door_id]
                                        % self.definition.ploidy));
                            }
                        }
                        if remaining.count_ones() == 1 {
                            let remaining = remaining.trailing_zeros() as usize;
                            for i in 0..self.definition.ploidy {
                                if connections[from_room_group + i][door_id] == !0 {
                                    println!(
                                        "Completed connection {}.{}->{}",
                                        from_room_group + i,
                                        door_id,
                                        to_room_group + remaining
                                    );
                                    connections[from_room_group + i][door_id] =
                                        to_room_group + remaining;
                                }
                            }
                        }
                    }
                };
            }
        }

        assert_eq!(room_digits.len(), connections.len());

        self.guess(room_digits, connections)?;

        Ok(())
    }

    fn explore(&mut self, plans: &[String]) -> Result<Vec<Vec<i8>>, Box<dyn std::error::Error>> {
        let explore_resp = self.requester.explore(plans.to_vec())?;
        println!(
            "Explored: {:?}",
            plans
                .iter()
                .zip_eq(explore_resp.results.iter())
                .map(|(p, r)| (p, r.iter().map(|d| d.to_string()).join("")))
                .collect::<Vec<(&String, String)>>()
        );

        // Store the result
        Ok(explore_resp.results)
    }

    fn guess(
        &self,
        room_digits: Vec<i8>,
        remaining_connections: Vec<Vec<usize>>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        for (i, (d, c)) in room_digits
            .iter()
            .zip_eq(remaining_connections.iter())
            .enumerate()
        {
            println!("{:?}: {:?} {:?}", i, d, c);
        }

        let mut connections = Vec::<GuessRequestConnection>::new();
        let mut remaining_connections = remaining_connections;

        for from_room in 0..remaining_connections.len() {
            for from_door in 0..6 {
                if remaining_connections[from_room][from_door] != !0 {
                    let to_room = remaining_connections[from_room][from_door];
                    if to_room == !0 {
                        return Err(format!(
                            "Failed to find connection from room {}.{}",
                            from_room, from_door
                        )
                        .into());
                    }
                    for to_door in 0..6 {
                        if remaining_connections[to_room][to_door] == from_room {
                            connections.push(GuessRequestConnection {
                                from: GuessRequestRoom {
                                    room: from_room,
                                    door: from_door,
                                },
                                to: GuessRequestRoom {
                                    room: to_room,
                                    door: to_door,
                                },
                            });
                            remaining_connections[to_room][to_door] = !0;
                            break;
                        }
                        if to_door == 5 {
                            return Err(format!(
                                "Failed to find connection from room {}.{} to room {}",
                                from_room, from_door, to_room
                            )
                            .into());
                        }
                    }
                    remaining_connections[from_room][from_door] = !0;
                }
            }
        }

        let guess_resp = self.requester.guess(GuessRequestMap {
            rooms: room_digits,
            starting_room: 0,
            connections,
        })?;

        println!("Guess response: {:?}", guess_resp);

        Ok(true)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <problem> [team_id]", args[0]);
        eprintln!("  problem: probatio, primus, secundus, tertius, quartus, quintus");
        eprintln!("  team_id: optional, if not provided, uses localhost:8080");
        std::process::exit(1);
    }

    let problem = args[1].clone();

    let requester = HttpRequester::new(args.get(2).cloned());

    println!("Problem: {}", problem);

    let mut solver = BruteForce::new(problem, requester)?;
    solver.solve()?;

    Ok(())
}
