use std::collections::{BTreeSet, HashMap, HashSet};
use std::env;

mod utils;

use itertools::Itertools;
use itertools::repeat_n;
use rand::seq::SliceRandom;

use icfpc2025::problem_definition::{ProblemDefinition, ProblemDefinitions};
use icfpc2025::request::*;
use icfpc2025::types::*;

use utils::{LabelObservation, UnionFind};

const QUERY_NUM: usize = 2;

struct RandomWalker<R> {
    definition: ProblemDefinition,
    requester: R,
}

impl<R: Requester> RandomWalker<R> {
    fn new(problem: String, requester: R) -> Result<Self, Box<dyn std::error::Error>> {
        let definitions = ProblemDefinitions::new();
        let definition = definitions.get(&problem).ok_or("Invalid problem name")?;

        let select_resp = requester.select(definition.name.clone(), None)?;
        println!("Selected problem: {:?}", select_resp);

        Ok(RandomWalker {
            definition: definition.clone(),
            requester,
        })
    }

    fn explore(&mut self, plans: &[String]) -> Result<Vec<Vec<i8>>, Box<dyn std::error::Error>> {
        let explore_resp = self.requester.explore(plans.to_vec())?;
        println!("Explored: {:?}", explore_resp);

        // Store the result
        Ok(explore_resp.results)
    }

    fn random_walk(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut rng = rand::rng();

        println!("Starting random walk for problem: {}", self.definition.name);
        println!(
            "Room count: {}",
            self.definition.size / self.definition.ploidy
        );

        // 長さ18*room_countのランダムな探索列を生成
        let mut exploration_plans = Vec::new();
        for _ in 0..QUERY_NUM {
            let mut exploration_plan = Vec::new();
            for i in 0..6 {
                for _ in 0..self.definition.max_plan_length / 6 {
                    exploration_plan.push(i.to_string());
                }
            }
            exploration_plan.shuffle(&mut rng);
            exploration_plans.push(exploration_plan.join(""));
        }

        println!("Generated exploration plans: {:?}", exploration_plans);

        // 探索実行
        match self.explore(&exploration_plans) {
            Ok(results) => {
                println!("Exploration completed successfully");
                println!("Number of results: {}", results.len());

                // 結果を分析
                let mut label_observation = self.analyze_results(&exploration_plans, &results)?;

                // 推測を実行
                let (node_label, start_index, graph) = self.guess(
                    self.definition.size / self.definition.ploidy,
                    &exploration_plans,
                    &results,
                    &mut label_observation,
                )?;
                let (node_label, start_index, graph) =
                    self.expand_ploidy(node_label, start_index, graph)?;
                let correct = self.submit_result(node_label, start_index, graph)?;
                if correct {
                    println!("✅ Guess successful!");
                } else {
                    println!("❌ Guess failed!");
                    return Err("Guess failed!".into());
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
    ) -> Result<LabelObservation, Box<dyn std::error::Error>> {
        println!("\n=== Analysis of Exploration Results ===");

        let mut label_observation =
            LabelObservation::new(self.definition.size / self.definition.ploidy);

        // 各探索結果を分析
        for (i, result) in results.iter().enumerate() {
            if i >= plans.len() {
                break;
            }

            for j in 0..plans[i].len() {
                let label = result[j] as usize;
                let door = plans[i][j..j + 1].parse::<usize>().unwrap();

                // 1ステップの観測
                if j + 1 < result.len() {
                    let next_label = result[j + 1] as usize;
                    label_observation.add_child(label, door, next_label);
                }

                // 2ステップの観測
                if j + 2 < result.len() && j + 1 < plans[i].len() {
                    let label2 = result[j + 1] as usize;
                    let door2 = plans[i][j + 1..j + 2].parse::<usize>().unwrap();
                    let next_label2 = result[j + 2] as usize;
                    label_observation.add_child_2step(label, door, label2, door2, next_label2);
                }

                // 3ステップの観測
                if j + 3 < result.len() && j + 2 < plans[i].len() {
                    let label2 = result[j + 1] as usize;
                    let door2 = plans[i][j + 1..j + 2].parse::<usize>().unwrap();
                    let label3 = result[j + 2] as usize;
                    let door3 = plans[i][j + 2..j + 3].parse::<usize>().unwrap();
                    let next_label3 = result[j + 3] as usize;
                    label_observation.add_child_3step(
                        &[label, label2, label3],
                        &[door, door2, door3],
                        next_label3,
                    );
                }
            }
        }

        println!("Label observation: {}", label_observation.get_sum_size());
        for i in 0..4 {
            println!("Label {} observation: {}", i, label_observation.get_size(i));
        }

        // is_unique である行動を出力する
        let unique_paths = label_observation.get_unique_paths();
        for (path, label) in unique_paths {
            if path.len() == 1 {
                println!(
                    "Label {} + Door {} : Label {} is unique",
                    path[0].label, path[0].door, label
                );
            } else if path.len() == 2 {
                println!(
                    "Label {} + Door {} + Label {} + Door {} : Label {} is unique",
                    path[0].label, path[0].door, path[1].label, path[1].door, label
                );
            } else if path.len() == 3 {
                println!(
                    "Label {} + Door {} + Label {} + Door {} + Label {} + Door {} : Label {} is unique",
                    path[0].label,
                    path[0].door,
                    path[1].label,
                    path[1].door,
                    path[2].label,
                    path[2].door,
                    label
                );
            }
        }

        Ok(label_observation)
    }

    fn guess(
        &self,
        node_count: usize,
        plans: &[String],
        results: &[Vec<i8>],
        label_observation: &mut LabelObservation,
    ) -> Result<(Vec<i8>, usize, Vec<Vec<usize>>), Box<dyn std::error::Error>> {
        println!("\n=== Analyzing Room Types by Label ===");

        let whole_size = QUERY_NUM * results[0].len();
        let mut uf = UnionFind::new(whole_size);
        let mut is_different_group = vec![vec![false; whole_size]; whole_size];

        for i in 0..whole_size {
            let label_i = results[i / results[0].len()][i % results[0].len()];
            for j in i + 1..whole_size {
                let label_j = results[j / results[0].len()][j % results[0].len()];
                if label_i != label_j {
                    is_different_group[i][j] = true;
                    is_different_group[j][i] = true;
                }
            }
        }

        for (k, plan) in plans.iter().enumerate() {
            for i in 0..plan.len() {
                let door_i = plan[i..i + 1].parse::<usize>().unwrap();
                let base_i = k * results[0].len();
                uf.set_edge(base_i + i, door_i, base_i + i + 1);
            }
        }

        let mut to_be_connected = QUERY_NUM * results[0].len() - node_count;

        // 各プランについてスタート地点は同じノード
        for k in 1..plans.len() {
            println!("Union: {} {}", 0, k * results[0].len());
            to_be_connected -= uf.union(0, k * results[0].len());
        }

        loop {
            let mut end = false;
            let merged_count = merge_group_by_unique_paths(results, &mut uf, label_observation);
            to_be_connected -= merged_count;
            if to_be_connected == 0 {
                break;
            }
            if merged_count == 0 {
                end = true;
            }
            println!("================");
            update_label_observation(results, &mut uf, label_observation);
            check_different_group(results, &mut uf, &mut is_different_group);

            let merged_count = merge_group_by_different_group(
                results,
                &mut uf,
                &is_different_group,
                label_observation,
            );
            to_be_connected -= merged_count;

            if (end && merged_count == 0) || to_be_connected == 0 {
                break;
            }
            if merged_count != 0 {
                update_label_observation(results, &mut uf, label_observation);
            }
        }

        for i in 0..uf.whole_size() {
            uf.find(i);
        }

        // サイズが大きい順に node_count 個の根を取得
        let mut roots = Vec::new();
        for i in 0..uf.whole_size() {
            if uf.find(i) == i {
                roots.push(i);
            }
        }

        // サイズが大きい順にソート
        roots.sort_by_key(|&i| uf.get_size(i));
        roots.reverse();
        roots.truncate(node_count);
        println!("Roots: {:?}", roots);

        // roots に対応する 0-index のインデックスを作る
        let mut root_to_index = HashMap::new();
        for (i, root) in roots.iter().enumerate() {
            root_to_index.insert(*root, i);
        }
        println!("Root to index: {:?}", root_to_index);
        println!("Start index: {}", uf.find(0));

        // スタート地点が roots に含まれていなかったら特定失敗
        if !roots.contains(&uf.find(0)) {
            return Err("Start index not found".into());
        }

        let start_index = root_to_index[&uf.find(0)];
        let mut node_label = vec![0; node_count];
        for (key, value) in root_to_index.iter() {
            node_label[*value] = results[*key / results[0].len()][*key % results[0].len()];
        }
        let mut graph = vec![vec![!0; 6]; node_count];
        let mut destination_labels = vec![vec![!0; 6]; node_count];

        {
            for i in 0..node_count {
                let edges = uf.get_edges(roots[i]);
                for j in 0..6 {
                    if root_to_index.contains_key(&edges[j]) {
                        let dst = root_to_index[&edges[j]];
                        graph[i][j] = dst;
                        destination_labels[i][j] =
                            results[dst / results[0].len()][dst % results[0].len()] as usize;
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
        }

        // println!("Union-Find: {:?}", uf);
        println!("start_index: {}", start_index);
        println!("node_label: {:?}", node_label);
        println!("destination_labels: {:?}", destination_labels);
        println!("graph: {:?}", graph);

        println!("To be connected: {}", to_be_connected);
        println!("{} {}", node_count, results[0].len());

        if graph.iter().any(|v| v.iter().any(|v| v == &!0)) {
            return Err("Graph is invalid".into());
        }

        return Ok((node_label, start_index, graph));
    }

    fn expand_ploidy(
        &mut self,
        node_label: Vec<i8>,
        start_index: usize,
        graph: Vec<Vec<usize>>,
    ) -> Result<(Vec<i8>, usize, Vec<Vec<usize>>), Box<dyn std::error::Error>> {
        // 1 倍では何もする必要はない
        if self.definition.ploidy == 1 {
            return Ok((node_label, start_index, graph));
        }

        if self.definition.ploidy > 2 {
            return Err("Not implemented yet!".into());
        }

        let sub_size = self.definition.size / self.definition.ploidy;

        // 各部屋から各部屋への最短経路
        let mut shortest_paths = vec![vec![None; sub_size]; sub_size];
        for i in 0..sub_size {
            let mut room_list = vec![(i, "".to_string())];
            while shortest_paths[i].iter().any(|v| v.is_none()) {
                let last_room_list = room_list;
                room_list = vec![];
                for (room, path) in last_room_list {
                    shortest_paths[i][room].get_or_insert(path.clone());
                    for (door, next_room) in graph[room].iter().enumerate() {
                        let next_path = path.clone() + &door.to_string();
                        if shortest_paths[i][*next_room].is_none() {
                            room_list.push((*next_room, next_path));
                        }
                    }
                }
            }
        }
        let shortest_paths = shortest_paths
            .into_iter()
            .map(|v| v.into_iter().map(|v| v.unwrap()).collect::<Vec<_>>())
            .collect::<Vec<_>>();

        println!("Shortest paths: {:?}", shortest_paths);

        // 全部屋を巡回する経路を生成する
        // 実質的に TSP なので改善の余地がある
        let mut visited_rooms = HashSet::new();
        let mut current_room = start_index;
        let mut paths = vec![];
        let mut room_order = vec![];

        while visited_rooms.len() < sub_size {
            visited_rooms.insert(current_room);
            room_order.push(current_room);

            // 最後は初期位置に戻る
            if visited_rooms.len() == sub_size {
                visited_rooms.remove(&start_index);
            }

            // 最寄りの未探索の部屋を探す
            let (next_room, next_path) = shortest_paths[current_room]
                .iter()
                .enumerate()
                .filter(|(room, _)| !visited_rooms.contains(room))
                .min_by_key(|(_, path)| path.len())
                .unwrap();
            paths.push(next_path);
            current_room = next_room;

            if current_room == start_index {
                break;
            }
        }

        assert_eq!(visited_rooms.len() + 1, sub_size);
        assert_eq!(room_order.len(), sub_size);
        assert_eq!(paths.len(), sub_size);

        // 巡回しながらラベルを変更するパスを生成する
        let painting_path = room_order
            .iter()
            .zip_eq(paths.iter())
            .map(|(room, path)| format!("[{}]{}", (node_label[*room] + 1) % 4, path))
            .join("");
        // 書き換えはコストを消費しない
        let painting_length = painting_path.len() - sub_size * 2;
        println!("Painting path: {}", painting_path);

        let mut unvisited_doors = BTreeSet::new();
        for room in 0..sub_size {
            for door in 0..6 {
                unvisited_doors.insert((room, door));
            }
        }

        // 未訪問のドアを訪問する経路を生成する
        // これも実質的に TSP なので改善の余地がある
        let mut planned_paths = vec![];
        while !unvisited_doors.is_empty() {
            let mut current_room = start_index;
            let mut current_path = "".to_string();
            let mut remaining_length = self.definition.max_plan_length - painting_length;
            while !unvisited_doors.is_empty() {
                let (target_room, target_door) = *unvisited_doors.first().unwrap();
                let next_path = &shortest_paths[current_room][target_room];
                if next_path.len() > remaining_length {
                    break;
                }
                remaining_length -= next_path.len();
                current_path += &next_path;
                for door in next_path.chars() {
                    let door = (door as u8 - b'0') as usize;
                    unvisited_doors.remove(&(current_room, door));
                    current_room = graph[current_room][door];
                }
                if remaining_length == 0 {
                    break;
                }
                remaining_length -= 1;
                current_path += &target_door.to_string();
                unvisited_doors.remove(&(current_room, target_door));
                current_room = graph[current_room][target_door];
            }
            planned_paths.push(painting_path.clone() + &current_path);
        }

        let planned_paths = planned_paths;
        println!("Planned paths: {:?}", planned_paths);

        let results = self.explore(&planned_paths)?;

        // ドア間の移動でのコピー間の移動を記録する
        let mut shifts = vec![vec![None; 6]; sub_size];
        for (plan, result) in planned_paths.iter().zip_eq(results.iter()) {
            let mut current_room = start_index;
            let mut current_shift = (4 + node_label[current_room] - result[painting_length]) % 4;

            let plan = plan[painting_path.len()..].to_string();
            let result = result[painting_length + 1..].to_vec();

            for (door, next_label) in plan.chars().zip_eq(result.iter()) {
                let door = (door as u8 - b'0') as usize;
                let next_room = graph[current_room][door];
                let next_shift = (4 + node_label[next_room] - next_label) % 4;
                shifts[current_room][door].get_or_insert((4 + current_shift - next_shift) % 4);
                current_room = next_room;
                current_shift = next_shift;
            }
        }

        let shifts = shifts
            .into_iter()
            .map(|v| v.into_iter().map(|v| v.unwrap()).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        println!("Shifts: {:?}", shifts);

        // シフトしたグラフを生成する
        let graph = repeat_n(graph, self.definition.ploidy)
            .enumerate()
            .flat_map(|(i, v)| {
                v.into_iter()
                    .zip_eq(shifts.iter())
                    .map(|(v, s)| {
                        v.into_iter()
                            .zip_eq(s)
                            .map(|(v, s)| v + (i + *s as usize) % self.definition.ploidy * sub_size)
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let node_label = repeat_n(node_label, self.definition.ploidy)
            .flatten()
            .collect::<Vec<_>>();

        println!("start_index: {}", start_index);
        println!("node_label: {:?}", node_label);
        println!("graph: {:?}", graph);

        Ok((node_label, start_index, graph))
    }

    fn submit_result(
        &self,
        node_label: Vec<i8>,
        start_index: usize,
        graph: Vec<Vec<usize>>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let node_count = node_label.len();

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
            rooms: node_label.clone(),
            starting_room: start_index,
            connections,
        })?;

        println!("Guess response: {:?}", guess_resp);

        Ok(true)
    }
}

// 3手分の行動で判別可能な部屋についてマージする
fn merge_group_by_unique_paths(
    results: &[Vec<i8>],
    uf: &mut UnionFind,
    label_observation: &LabelObservation,
) -> usize {
    let mut merged_count = 0;
    let unique_paths = label_observation.get_unique_paths();
    let length = results[0].len();
    for (actions, label) in unique_paths {
        for (k1, result1) in results.iter().enumerate() {
            for (i1, r1) in result1.iter().enumerate() {
                if actions[0].label != *r1 as usize {
                    continue;
                }
                let path1 = uf.get_path(k1 * length + i1, &actions);
                let mut ok = true;
                for j in 0..path1.len() - 1 {
                    if path1[j] == !0 {
                        ok = false;
                        break;
                    }
                    if results[path1[j] / length][path1[j] % length] as usize
                        != actions[j + 1].label
                    {
                        ok = false;
                        break;
                    }
                }
                let last_path1 = path1.last().unwrap();
                if *last_path1 == !0
                    || results[*last_path1 / length][*last_path1 % length] as usize != label
                {
                    ok = false;
                }
                if !ok {
                    continue;
                }
                for (k2, result2) in results.iter().enumerate().skip(k1) {
                    let start_idx = if k1 == k2 { i1 + 1 } else { 0 };
                    for (i2, r2) in result2.iter().enumerate().skip(start_idx) {
                        if actions[0].label != *r2 as usize {
                            continue;
                        }
                        if uf.find(k1 * length + i1) == uf.find(k2 * length + i2) {
                            continue;
                        }
                        let path2 = uf.get_path(k2 * length + i2, &actions);
                        let mut ok = true;
                        for j in 0..path2.len() - 1 {
                            if path2[j] == !0 {
                                ok = false;
                                break;
                            }
                            if results[path2[j] / length][path2[j] % length] as usize
                                != actions[j + 1].label
                            {
                                ok = false;
                                break;
                            }
                        }
                        let last_path2 = path2.last().unwrap();
                        if *last_path2 == !0
                            || results[*last_path2 / length][*last_path2 % length] as usize != label
                        {
                            ok = false;
                        }
                        if !ok {
                            continue;
                        }
                        println!("Union: {} {}", k1 * length + i1, k2 * length + i2);
                        merged_count += uf.union(k1 * length + i1, k2 * length + i2);
                    }
                }
            }
        }
    }
    merged_count
}

// 判明している接続情報を元に label_observation を更新
fn update_label_observation(
    results: &[Vec<i8>],
    uf: &mut UnionFind,
    label_observation: &mut LabelObservation,
) {
    let length = results[0].len();
    // label_observation を更新
    for i in 0..results.len() * length {
        // 既に結合されているものはスキップ
        if uf.find(i) != i {
            continue;
        }
        let label0 = results[i / length][i % length] as usize;
        for d0 in 0..6 {
            let pos1 = uf.get_next(i, d0);
            if pos1 == !0 {
                continue;
            }
            let label1 = results[pos1 / length][pos1 % length] as usize;
            label_observation.add_child(label0, d0, label1);
            for d1 in 0..6 {
                let pos2 = uf.get_next(pos1, d1);
                if pos2 == !0 {
                    continue;
                }
                let label2 = results[pos2 / length][pos2 % length] as usize;
                label_observation.add_child_2step(label0, d0, label1, d1, label2);
                for d2 in 0..6 {
                    let pos3 = uf.get_next(pos2, d2);
                    if pos3 == !0 {
                        continue;
                    }
                    let label3 = results[pos3 / length][pos3 % length] as usize;
                    label_observation.add_child_3step(
                        &[label0, label1, label2],
                        &[d0, d1, d2],
                        label3,
                    );
                }
            }
        }
    }
}

fn check_different_group(
    results: &[Vec<i8>],
    uf: &mut UnionFind,
    is_different_group: &mut [Vec<bool>],
) {
    fn check_different_group_impl(
        results: &[Vec<i8>],
        uf: &mut UnionFind,
        is_different_group: &[Vec<bool>],
        idx0: usize,
        idx1: usize,
        depth: usize,
    ) -> bool {
        if depth == 0 {
            return false;
        }
        let length = results[0].len();
        for d0 in 0..6 {
            let pos1_1 = uf.get_next(idx0, d0);
            if pos1_1 == !0 {
                continue;
            }
            let pos2_1 = uf.get_next(idx1, d0);
            if pos2_1 == !0 {
                continue;
            }
            if is_different_group[pos1_1][pos2_1] {
                return true;
            }
            let label1_1 = results[pos1_1 / length][pos1_1 % length] as usize;
            let label2_1 = results[pos2_1 / length][pos2_1 % length] as usize;
            if label1_1 != label2_1 {
                return true;
            }
            if check_different_group_impl(
                results,
                uf,
                is_different_group,
                pos1_1,
                pos2_1,
                depth - 1,
            ) {
                return true;
            }
        }
        false
    }

    loop {
        let mut updated = false;
        let whole_size = uf.whole_size();
        for i in 0..whole_size {
            if uf.find(i) != i {
                continue;
            }
            for j in i + 1..whole_size {
                if is_different_group[i][j] {
                    continue;
                }
                if uf.find(j) != j {
                    continue;
                }
                if check_different_group_impl(results, uf, is_different_group, i, j, 3) {
                    is_different_group[i][j] = true;
                    is_different_group[j][i] = true;
                    updated = true;
                }
            }
        }
        if !updated {
            break;
        }
    }
}

fn merge_group_by_different_group(
    results: &[Vec<i8>],
    uf: &mut UnionFind,
    is_different_group: &[Vec<bool>],
    label_observation: &LabelObservation,
) -> usize {
    let mut merged_count = 0;

    let unique_paths = label_observation.get_unique_paths();
    let length = results[0].len();
    let mut div_groups = vec![vec![Vec::<usize>::new(); 6]; 4];
    for (actions, label) in unique_paths {
        for (k1, result1) in results.iter().enumerate() {
            for (i1, r1) in result1.iter().enumerate() {
                if uf.find(k1 * length + i1) != k1 * length + i1 {
                    continue;
                }
                if actions[0].label != *r1 as usize {
                    continue;
                }
                let path1 = uf.get_path(k1 * length + i1, &actions);
                let mut ok = true;
                for j in 0..path1.len() - 1 {
                    if path1[j] == !0 {
                        ok = false;
                        break;
                    }
                    if results[path1[j] / length][path1[j] % length] as usize
                        != actions[j + 1].label
                    {
                        ok = false;
                        break;
                    }
                }
                let last_path1 = path1.last().unwrap();
                if *last_path1 == !0
                    || results[*last_path1 / length][*last_path1 % length] as usize != label
                {
                    ok = false;
                }
                if !ok {
                    continue;
                }
                div_groups[actions[0].label][actions[0].door].push(k1 * length + i1);
            }
        }
    }

    for label_group in div_groups.iter() {
        for door_group in label_group.iter() {
            if door_group.is_empty() {
                continue;
            }
            for (i, diffs) in is_different_group.iter().enumerate() {
                if uf.find(i) != i {
                    continue;
                }
                let mut target = !0;
                for j in door_group.iter() {
                    if uf.find(i) == uf.find(*j) {
                        target = !0;
                        break;
                    }
                    if !diffs[*j] {
                        if target == !0 {
                            target = *j;
                        } else {
                            target = !0;
                            break;
                        }
                    }
                }
                if target != !0 {
                    println!("Group Union: {} {}", i, target);
                    merged_count += uf.union(i, target);
                }
            }
        }
    }

    // 残っているグループをサイズが大きい順に並べる
    let mut roots = Vec::new();
    for i in 0..uf.whole_size() {
        if uf.find(i) == i {
            roots.push(i);
        }
    }

    // サイズが大きい順にソート
    roots.sort_by_key(|&i| uf.get_size(i));
    roots.reverse();

    let mut leaders = vec![Vec::<usize>::new(); 4];
    let mut remains = vec![Vec::<usize>::new(); 4];
    let num_rooms = label_observation.get_room_count();

    for r in roots {
        let label = results[r / results[0].len()][r % results[0].len()] as usize;
        if leaders[label].len() < num_rooms[label] {
            let mut ok = true;
            for i in leaders[label].iter() {
                if !is_different_group[r][*i] {
                    ok = false;
                    break;
                }
            }
            if ok {
                leaders[label].push(r);
            } else {
                remains[label].push(r);
            }
        } else {
            remains[label].push(r);
        }
    }

    for i in 0..4 {
        if leaders[i].len() < num_rooms[i] {
            continue;
        }
        for j in remains[i].iter() {
            let mut target = !0;
            for l in leaders[i].iter() {
                if !is_different_group[*j][*l] {
                    if target == !0 {
                        target = *l;
                    } else {
                        target = !0;
                        break;
                    }
                }
            }
            if target != !0 {
                println!("Group Union: {} {}", *j, target);
                merged_count += uf.union(*j, target);
            }
        }
    }
    merged_count
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

    let requester = HttpRequester::new(args.get(2).cloned());

    // Determine base URL
    let base_url = if args.len() > 2 {
        "https://31pwr5t6ij.execute-api.eu-west-2.amazonaws.com".to_string()
    } else {
        "http://localhost:8080".to_string()
    };

    println!("Problem: {}", problem);
    println!("Team ID: {}", team_id);
    println!("Base URL: {}", base_url);

    let mut walker = RandomWalker::new(problem, requester)?;
    walker.random_walk()?;

    Ok(())
}
