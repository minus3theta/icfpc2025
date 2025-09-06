use std::collections::{HashMap, HashSet};
use std::env;

use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
struct UnionFind {
    parent: Vec<usize>,
    size: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: vec![!0; n],
            size: vec![1; n],
        }
    }
}

impl UnionFind {
    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] == !0 {
            x
        } else {
            self.parent[x] = self.find(self.parent[x]);
            self.parent[x]
        }
    }

    fn union(&mut self, x: usize, y: usize) {
        let root_x = self.find(x);
        let root_y = self.find(y);
        if root_x == root_y {
            return;
        }
        self.parent[root_y] = root_x;
        self.size[root_x] += self.size[root_y];
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SelectRequest {
    id: String,
    #[serde(rename = "problemName")]
    problem_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SelectResponse {
    #[serde(rename = "problemName")]
    problem_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExploreRequest {
    id: String,
    plans: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExploreResponse {
    results: Vec<Vec<i8>>,
    #[serde(rename = "queryCount")]
    query_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct GuessRequestMap {
    rooms: Vec<i8>,
    #[serde(rename = "startingRoom")]
    starting_room: usize,
    connections: Vec<GuessRequestConnection>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GuessRequestConnection {
    from: GuessRequestRoom,
    to: GuessRequestRoom,
}

#[derive(Debug, Serialize, Deserialize)]
struct GuessRequestRoom {
    room: usize,
    door: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct GuessRequest {
    id: String,
    map: GuessRequestMap,
}

#[derive(Debug, Serialize, Deserialize)]
struct GuessResponse {
    correct: bool,
}

struct RandomWalker {
    id: String,
    problem: String,
    room_count: u32,
    base_url: String,
}

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct Action {
    label: i8,
    door: usize,
}

impl Action {
    fn new(label: i8, door: usize) -> Self {
        Self { label, door }
    }
}

impl RandomWalker {
    fn new(
        problem: String,
        team_id: String,
        base_url: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let room_count = match problem.as_str() {
            "probatio" => 3,
            "primus" => 6,
            "secundus" => 12,
            "tertius" => 18,
            "quartus" => 24,
            "quintus" => 30,
            _ => return Err("Invalid problem name".into()),
        };

        let client = reqwest::blocking::Client::new();

        // Select problem
        let select_req = SelectRequest {
            id: team_id.clone(),
            problem_name: problem.clone(),
        };
        let response = client
            .post(format!("{}/select", base_url))
            .json(&select_req)
            .send()?;

        let response_text = response.text()?;
        println!("Select response text: {}", response_text);

        let select_resp: SelectResponse = serde_json::from_str(&response_text)?;
        println!("Selected problem: {:?}", select_resp);

        Ok(RandomWalker {
            id: team_id.clone(),
            problem,
            room_count,
            base_url,
        })
    }

    fn explore(&mut self, plans: &[String]) -> Result<Vec<Vec<i8>>, Box<dyn std::error::Error>> {
        let client = reqwest::blocking::Client::new();
        let explore_req = ExploreRequest {
            id: self.id.clone(),
            plans: plans.to_vec(),
        };

        let response = client
            .post(format!("{}/explore", self.base_url))
            .json(&explore_req)
            .send()?;

        let response_text = response.text()?;
        println!("Explore response text: {}", response_text);

        let explore_resp: ExploreResponse = serde_json::from_str(&response_text)?;
        println!("Explored: {:?}", explore_resp);

        // Store the result
        Ok(explore_resp.results)
    }

    fn random_walk(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut rng = rand::rng();
        let exploration_length = 18 * self.room_count as usize;

        println!("Starting random walk for problem: {}", self.problem);
        println!("Room count: {}", self.room_count);
        println!("Exploration length: {}", exploration_length);

        // 長さ18*room_countのランダムな探索列を生成
        let mut exploration_plans = Vec::new();
        for _ in 0..exploration_length {
            let door = rng.random_range(0..6);
            exploration_plans.push(door.to_string());
        }

        let exploration_plans = vec![exploration_plans.join("")];

        println!("Generated exploration plans: {:?}", exploration_plans);

        // 探索実行
        match self.explore(&exploration_plans) {
            Ok(results) => {
                println!("Exploration completed successfully");
                println!("Number of results: {}", results.len());

                // 結果を分析
                let room_door_destinations = self.analyze_results(&exploration_plans, &results)?;

                // 推測を実行
                let correct = self.guess(
                    self.room_count as usize,
                    &exploration_plans,
                    &results,
                    &room_door_destinations,
                )?;
                if correct {
                    println!("✅ Guess successful!");
                } else {
                    println!("❌ Guess failed!");
                }
            }
            Err(e) => {
                println!("Exploration failed: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }

    fn analyze_results(
        &self,
        plans: &[String],
        results: &[Vec<i8>],
    ) -> Result<HashMap<Action, HashSet<i8>>, Box<dyn std::error::Error>> {
        println!("\n=== Analysis of Exploration Results ===");

        // 部屋のラベルとドアの組について、行先がとり得るラベルの値の集合を記録
        let mut room_door_destinations: HashMap<Action, HashSet<i8>> = HashMap::new();
        let mut room_door_destinations2: HashMap<(Action, Action), HashSet<i8>> = HashMap::new();

        // 各探索結果を分析
        for (i, result) in results.iter().enumerate() {
            if i >= plans.len() {
                break;
            }

            for j in 0..plans[i].len() {
                let label = result[j];
                let door = plans[i][j..j + 1].parse::<usize>().unwrap();
                let next_label = result[j + 1];
                room_door_destinations
                    .entry(Action::new(label, door))
                    .or_default()
                    .insert(next_label);
            }
            for j in 0..plans[i].len() - 1 {
                let label = result[j];
                let door = plans[i][j..j + 1].parse::<usize>().unwrap();
                let next_label = result[j + 1];
                let next_door = plans[i][j + 1..j + 2].parse::<usize>().unwrap();
                let next_next_label = result[j + 2];
                room_door_destinations2
                    .entry((Action::new(label, door), Action::new(next_label, next_door)))
                    .or_default()
                    .insert(next_next_label);
            }
        }

        // 結果を出力
        println!("\nRoom Label + Door -> Possible Destination Labels:");
        let mut sorted_keys: Vec<_> = room_door_destinations.keys().collect();
        sorted_keys.sort();

        for &action in sorted_keys.iter() {
            if let Some(destinations) = room_door_destinations.get(action) {
                let mut sorted_destinations: Vec<_> = destinations.iter().collect();
                sorted_destinations.sort();
                println!(
                    "  Room Label {} + Door {} -> {:?}",
                    action.label, action.door, sorted_destinations
                );
            }
        }

        let mut sorted_keys2: Vec<_> = room_door_destinations2.keys().collect();
        sorted_keys2.sort();
        for &actions in sorted_keys2.iter() {
            if let Some(destinations) = room_door_destinations2.get(actions) {
                let mut sorted_destinations: Vec<_> = destinations.iter().collect();
                sorted_destinations.sort();
                println!(
                    "  Room Label {} + Door {} + Next Label {} + Next Door {} -> {:?}",
                    actions.0.label,
                    actions.0.door,
                    actions.1.label,
                    actions.1.door,
                    sorted_destinations
                );
            }
        }

        // 統計情報
        println!("\n=== Statistics ===");
        println!(
            "Total room-door combinations found: {}",
            room_door_destinations.len()
        );

        let mut total_destinations = 0;
        for destinations in room_door_destinations.values() {
            total_destinations += destinations.len();
        }
        println!("Total destination possibilities: {}", total_destinations);

        if !room_door_destinations.is_empty() {
            let avg_destinations = total_destinations as f64 / room_door_destinations.len() as f64;
            println!(
                "Average destinations per room-door combination: {:.2}",
                avg_destinations
            );
        }

        Ok(room_door_destinations)
    }

    fn guess(
        &self,
        node_count: usize,
        plans: &[String],
        results: &[Vec<i8>],
        room_door_destinations: &HashMap<Action, HashSet<i8>>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let mut num_rooms = [0; 4];

        for (action, destinations) in room_door_destinations.iter() {
            num_rooms[action.label as usize] =
                num_rooms[action.label as usize].max(destinations.len());
        }

        let sum_num_rooms = num_rooms.iter().sum::<usize>();

        if sum_num_rooms != node_count {
            println!(
                "Total room types ({}) does not match node count ({})",
                sum_num_rooms, node_count
            );
            return Ok(false);
        }
        println!("\n=== Analyzing Room Types by Label ===");

        let mut uf = UnionFind::new(results[0].len());

        let mut to_be_connected = results[0].len() - node_count;

        for (k, plan) in plans.iter().enumerate() {
            if results.len() <= k {
                break;
            }
            // ラベルとドアの組み合わせに対し、行先の種類数がラベルに対応する部屋の数に一致するものについて、
            // 同じラベル・同じ行先の部屋は同じものとして統合する
            for i in 0..plan.len() - 1 {
                let door_i = plans[k][i..i + 1].parse::<usize>().unwrap();
                let label_i = results[k][i];
                if !room_door_destinations.contains_key(&Action::new(label_i, door_i)) {
                    continue;
                }
                let destinations = room_door_destinations
                    .get(&Action::new(label_i, door_i))
                    .unwrap();
                if destinations.len() != num_rooms[label_i as usize] {
                    continue;
                }
                let destination_i = results[k][i + 1];
                for j in i + 1..plans[k].len() - 1 {
                    let door_j = plans[k][j..j + 1].parse::<usize>().unwrap();
                    let label_j = results[k][j];
                    let destination_j = results[k][j + 1];
                    if label_i != label_j {
                        continue;
                    }
                    // ラベルに対応する部屋が1個しかなければ、同じラベルのものは同じ部屋で確定
                    if num_rooms[label_i as usize] > 1 {
                        // 選んだドアとその行先が同じなら同じ部屋とみなす
                        if door_i != door_j {
                            continue;
                        }
                        if destination_i != destination_j {
                            continue;
                        }
                    }
                    if uf.find(i) == uf.find(j) {
                        continue;
                    }

                    uf.union(i, j);
                    to_be_connected -= 1;
                }
            }
        }

        loop {
            if to_be_connected == 0 {
                break;
            }
            let mut updated = false;
            for plan in plans.iter() {
                for i in 0..plan.len() - 1 {
                    let door_i = plan[i..i + 1].parse::<usize>().unwrap();
                    let root_i = uf.find(i);
                    let root_ni = uf.find(i + 1);
                    for j in i + 1..plan.len() - 1 {
                        let door_j = plan[j..j + 1].parse::<usize>().unwrap();
                        let root_j = uf.find(j);
                        let root_nj = uf.find(j + 1);
                        if root_i != root_j {
                            continue;
                        }
                        if door_i != door_j {
                            continue;
                        }
                        if root_ni == root_nj {
                            continue;
                        }
                        uf.union(root_ni, root_nj);
                        updated = true;
                        to_be_connected -= 1;
                    }
                }
            }
            if !updated {
                break;
            }
        }

        for i in 0..uf.parent.len() {
            uf.find(i);
        }

        // サイズが大きい順に node_count 個の根を取得
        let mut roots = Vec::new();
        for i in 0..uf.parent.len() {
            if uf.parent[i] == !0 {
                roots.push(i);
            }
        }

        // サイズが大きい順にソート
        roots.sort_by_key(|&i| uf.size[i]);
        roots.reverse();
        roots.truncate(node_count);
        println!("Roots: {:?}", roots);

        // roots に対応する 0-index のインデックスを作る
        let mut root_to_index = HashMap::new();
        for (i, root) in roots.iter().enumerate() {
            root_to_index.insert(*root, i);
        }
        println!("Root to index: {:?}", root_to_index);

        // スタート地点が roots に含まれていなかったら特定失敗
        if !roots.contains(&uf.find(0)) {
            return Ok(false);
        }

        let start_index = root_to_index[&uf.find(0)];
        let mut node_label = vec![0; node_count];
        for (key, value) in root_to_index.iter() {
            node_label[*value] = results[0][*key];
        }
        let mut graph = vec![vec![!0; 6]; node_count];
        let mut destination_labels = vec![vec![!0; 6]; node_count];

        loop {
            for k in 0..plans.len() {
                for i in 0..plans[k].len() - 1 {
                    let door_i = plans[k][i..i + 1].parse::<usize>().unwrap();
                    let root_i = uf.find(i);
                    if root_to_index.contains_key(&root_i) {
                        destination_labels[root_to_index[&root_i]][door_i] =
                            results[k][i + 1] as usize;
                    }
                }
            }

            for plan in plans.iter() {
                for i in 0..plan.len() {
                    let door_i = plan[i..i + 1].parse::<usize>().unwrap();
                    let root_i = uf.find(i);
                    let root_ni = uf.find(i + 1);
                    if root_to_index.contains_key(&root_i) && root_to_index.contains_key(&root_ni) {
                        graph[root_to_index[&root_i]][door_i] = root_to_index[&root_ni];
                    }
                }
            }

            // 入出次数のマッチング
            for i in 0..node_count {
                for j in 0..node_count {
                    if i == j {
                        continue;
                    }
                    let mut count_i = 0;
                    let mut count_j = 0;
                    for k in 0..6 {
                        if graph[i][k] == j {
                            count_i += 1;
                        }
                        if graph[j][k] == i {
                            count_j += 1;
                        }
                    }
                    if count_i >= count_j {
                        continue;
                    }
                    let rest = count_j - count_i;
                    let mut loose_candidates = Vec::new();
                    let mut has_label = true;
                    for k in 0..6 {
                        if graph[i][k] == !0 {
                            loose_candidates.push(k);
                            if destination_labels[i][k] == !0 {
                                has_label = false;
                            }
                        }
                    }
                    // 行先不明のドアの数と不足数が一致するなら補完
                    if rest == loose_candidates.len() {
                        for k in loose_candidates {
                            graph[i][k] = j;
                        }
                        continue;
                    }
                    let mut candidates = Vec::new();
                    // すべての行先のラベルが判明していて、条件に合うドアの数が不足分に足りてたら補完
                    if has_label {
                        for k in 0..6 {
                            if graph[i][k] == !0
                                && destination_labels[i][k] == node_label[j] as usize
                            {
                                candidates.push(k);
                            }
                        }
                    }
                    if candidates.len() == rest {
                        for k in candidates {
                            graph[i][k] = j;
                        }
                        continue;
                    }
                }
            }

            let mut updated = false;

            // 確定した頂点の確定した行先について、状態が不明なものがあれば設定する
            for plan in plans.iter() {
                for i in 0..plan.len() {
                    let door_i = plan[i..i + 1].parse::<usize>().unwrap();
                    let root_i = uf.find(i);
                    if root_to_index.contains_key(&root_i) {
                        let index_i = root_to_index[&root_i];
                        if graph[index_i][door_i] != !0 {
                            let index_j = graph[index_i][door_i];
                            let root_j = uf.find(i + 1);
                            if !root_to_index.contains_key(&root_j) {
                                uf.size[roots[index_j]] += uf.size[root_j];
                                uf.parent[root_j] = roots[index_j];
                                updated = true;
                            }
                        }
                    }
                }
            }
            // 頂点のラベルと選んだドアの組み合わせに対して、確定済みの行先と元の頂点の組の候補を列挙する
            let mut destination_candidates = HashMap::new();
            for (k, plan) in plans.iter().enumerate() {
                for i in 0..plan.len() {
                    let door_i = plan[i..i + 1].parse::<usize>().unwrap();
                    let root_i = uf.find(i);
                    if root_to_index.contains_key(&root_i) {
                        let index_i = root_to_index[&root_i];
                        let root_ni = uf.find(i + 1);
                        if root_to_index.contains_key(&root_ni) {
                            let index_ni = root_to_index[&root_ni];
                            destination_candidates
                                .entry((results[k][i], door_i))
                                .or_insert_with(HashSet::new)
                                .insert((index_ni, index_i));
                        }
                    }
                }
            }
            // 現在の状態で、ラベルとドアの組み合わせに対してあり得る行先を列挙し、
            // 部屋数と同じ個数の候補があるところについて、その情報を用いて場所を確定させる
            for k in 0..plans.len() {
                for i in 0..plans[k].len() {
                    let door_i = plans[k][i..i + 1].parse::<usize>().unwrap();
                    let root_i = uf.find(i);
                    if root_to_index.contains_key(&root_i) {
                        continue;
                    }
                    let root_ni = uf.find(i + 1);
                    if !root_to_index.contains_key(&root_ni) {
                        continue;
                    }
                    let label_i = results[k][i];
                    if destination_candidates.contains_key(&(label_i, door_i)) {
                        let candidates = &destination_candidates[&(label_i, door_i)];
                        if candidates.len() == num_rooms[label_i as usize] {
                            for candidate in candidates {
                                if candidate.0 == root_to_index[&root_ni] {
                                    uf.size[candidate.1] += uf.size[root_i];
                                    uf.parent[root_i] = candidate.1;
                                    updated = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            if !updated {
                break;
            }
        }

        println!("Union-Find: {:?}", uf);
        println!("start_index: {}", start_index);
        println!("node_label: {:?}", node_label);
        println!("destination_labels: {:?}", destination_labels);
        println!("graph: {:?}", graph);
        println!("To be connected: {}", to_be_connected);
        println!("{} {}", node_count, results[0].len());

        let mut remaining_connections = graph.clone();
        let mut connections = Vec::<GuessRequestConnection>::new();

        for from_room in 0..node_count {
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
                    remaining_connections[from_room][from_door] = !0;
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
                }
            }
        }

        let client = reqwest::blocking::Client::new();
        let guess_req = GuessRequest {
            id: self.id.clone(),
            map: GuessRequestMap {
                rooms: node_label.clone(),
                starting_room: start_index,
                connections,
            },
        };

        let response = client
            .post(format!("{}/guess", self.base_url))
            .json(&guess_req)
            .send()?;

        let response_text = response.text()?;
        println!("Guess response text: {}", response_text);

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
    let team_id = args.get(2).cloned().unwrap_or_else(|| "hoge".to_string());

    // Determine base URL
    let base_url = if args.len() > 2 {
        "https://31pwr5t6ij.execute-api.eu-west-2.amazonaws.com".to_string()
    } else {
        "http://localhost:8080".to_string()
    };

    println!("Problem: {}", problem);
    println!("Team ID: {}", team_id);
    println!("Base URL: {}", base_url);

    let mut walker = RandomWalker::new(problem, team_id, base_url)?;
    walker.random_walk()?;

    Ok(())
}
