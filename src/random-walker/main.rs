use std::collections::{HashMap, HashSet};
use std::env;

use rand::seq::SliceRandom;

#[path = "../request.rs"]
mod request;

use request::*;

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

struct RandomWalker {
    problem: String,
    room_count: u32,
    requester: Requester,
}

struct LabelObservationNode {
    child: HashMap<usize, HashMap<i8, LabelObservationNode>>,
}

impl LabelObservationNode {
    fn new() -> Self {
        Self {
            child: HashMap::new(),
        }
    }

    fn add_child(&mut self, door: usize, label: i8) {
        if !self.child.contains_key(&door) || !self.child[&door].contains_key(&label) {
            self.child
                .entry(door)
                .or_default()
                .insert(label, LabelObservationNode::new());
        }
    }

    fn get_child(&self, door: usize, label: i8) -> Option<&LabelObservationNode> {
        self.child.get(&door).and_then(|c| c.get(&label))
    }

    fn get_child_mut(&mut self, door: usize, label: i8) -> Option<&mut LabelObservationNode> {
        self.child.get_mut(&door).and_then(|c| c.get_mut(&label))
    }

    fn get_size(&self) -> usize {
        let mut res = 1;
        for c in self.child.values() {
            let mut sum = 0;
            for c in c.values() {
                sum += c.get_size();
            }
            res = res.max(sum);
        }
        res
    }

    fn is_unique(&self) -> bool {
        self.get_size() == 1
    }

    fn is_max_selection(&self, door: usize) -> bool {
        let mut max_size = 0;
        let mut cur_size = 0;
        for (k, v) in self.child.iter() {
            let mut sum = 0;
            for c in v.values() {
                sum += c.get_size();
            }
            max_size = max_size.max(sum);
            if k == &door {
                cur_size = sum;
            }
        }
        cur_size == max_size
    }
}

struct LabelObservation {
    nodes: [LabelObservationNode; 4],
}

impl LabelObservation {
    fn new() -> Self {
        Self {
            nodes: [
                LabelObservationNode::new(),
                LabelObservationNode::new(),
                LabelObservationNode::new(),
                LabelObservationNode::new(),
            ],
        }
    }

    fn get_size(&self, label: i8) -> usize {
        self.nodes[label as usize].get_size()
    }

    fn get_sum_size(&self) -> usize {
        self.nodes.iter().map(|n| n.get_size()).sum()
    }

    fn add_child(&mut self, label: i8, door: usize, destination_label: i8) {
        self.nodes[label as usize].add_child(door, destination_label);
    }

    fn add_child_2step(
        &mut self,
        label0: i8,
        door0: usize,
        label1: i8,
        door1: usize,
        destination_label: i8,
    ) {
        self.add_child(label0, door0, label1);
        self.nodes[label0 as usize]
            .get_child_mut(door0, label1)
            .unwrap()
            .add_child(door1, destination_label);
    }

    fn add_child_3step(&mut self, label: &[i8], door: &[usize], destination_label: i8) {
        self.add_child_2step(label[0], door[0], label[1], door[1], label[2]);
        self.nodes[label[0] as usize]
            .get_child_mut(door[0], label[1])
            .unwrap()
            .get_child_mut(door[1], label[2])
            .unwrap()
            .add_child(door[2], destination_label);
    }

    fn is_unique_1step(&self, label: i8, door: usize, destination_label: i8) -> bool {
        // このドアを選んだ場合の行先が、このラベルに対応する部屋の中で最も多くの行先を持つドアであり、かつその行先が一意かどうかをチェック
        self.nodes[label as usize].is_max_selection(door)
            && self.nodes[label as usize]
                .get_child(door, destination_label)
                .is_some()
            && self.nodes[label as usize]
                .get_child(door, destination_label)
                .unwrap()
                .is_unique()
    }

    fn is_unique_2step(
        &self,
        label0: i8,
        door0: usize,
        label1: i8,
        door1: usize,
        destination_label: i8,
    ) -> bool {
        // このドアを選んだ場合の行先が、このラベルに対応する部屋の中で最も多くの行先を持つドアであり、かつその行先が一意かどうかをチェック
        if !self.nodes[label0 as usize].is_max_selection(door0) {
            return false;
        }
        let child = self.nodes[label0 as usize].get_child(door0, label1);
        if child.is_none() {
            return false;
        }
        let child = child.unwrap();
        if !child.is_max_selection(door1) {
            return false;
        }
        let child = child.get_child(door1, destination_label);
        if child.is_none() {
            return false;
        }
        let child = child.unwrap();
        child.is_unique()
    }

    fn is_unique_3step(&self, label: &[i8], door: &[usize], destination_label: i8) -> bool {
        // このドアを選んだ場合の行先が、このラベルに対応する部屋の中で最も多くの行先を持つドアであり、かつその行先が一意かどうかをチェック
        if !self.nodes[label[0] as usize].is_max_selection(door[0]) {
            return false;
        }
        let child0 = self.nodes[label[0] as usize].get_child(door[0], label[1]);
        if child0.is_none() {
            return false;
        }
        let child0 = child0.unwrap();
        if !child0.is_max_selection(door[1]) {
            return false;
        }
        let child1 = child0.get_child(door[1], label[2]);
        if child1.is_none() {
            return false;
        }
        let child1 = child1.unwrap();
        if !child1.is_max_selection(door[2]) {
            return false;
        }
        let child2 = child1.get_child(door[2], destination_label);
        if child2.is_none() {
            return false;
        }
        let child2 = child2.unwrap();
        child2.is_unique()
    }
}

impl RandomWalker {
    fn new(problem: String, requester: Requester) -> Result<Self, Box<dyn std::error::Error>> {
        let room_count = match problem.as_str() {
            "probatio" => 3,
            "primus" => 6,
            "secundus" => 12,
            "tertius" => 18,
            "quartus" => 24,
            "quintus" => 30,
            _ => return Err("Invalid problem name".into()),
        };

        let select_resp = requester.select(problem.clone())?;
        println!("Selected problem: {:?}", select_resp);

        Ok(RandomWalker {
            problem,
            room_count,
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

        println!("Starting random walk for problem: {}", self.problem);
        println!("Room count: {}", self.room_count);

        // 長さ18*room_countのランダムな探索列を生成
        let mut exploration_plans = Vec::new();
        for i in 0..6 {
            for _ in 0..3 * self.room_count {
                exploration_plans.push(i.to_string());
            }
        }
        exploration_plans.shuffle(&mut rng);

        let exploration_plans = vec![exploration_plans.join("")];

        println!("Generated exploration plans: {:?}", exploration_plans);

        // 探索実行
        match self.explore(&exploration_plans) {
            Ok(results) => {
                println!("Exploration completed successfully");
                println!("Number of results: {}", results.len());

                // 結果を分析
                let label_observation = self.analyze_results(&exploration_plans, &results)?;

                // 推測を実行
                let correct = self.guess(
                    self.room_count as usize,
                    &exploration_plans,
                    &results,
                    &label_observation,
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
    ) -> Result<LabelObservation, Box<dyn std::error::Error>> {
        println!("\n=== Analysis of Exploration Results ===");

        let mut label_observation = LabelObservation::new();

        // 各探索結果を分析
        for (i, result) in results.iter().enumerate() {
            if i >= plans.len() {
                break;
            }

            for j in 0..plans[i].len() {
                let label = result[j];
                let door = plans[i][j..j + 1].parse::<usize>().unwrap();

                // 1ステップの観測
                if j + 1 < result.len() {
                    let next_label = result[j + 1];
                    label_observation.add_child(label, door, next_label);
                }

                // 2ステップの観測
                if j + 2 < result.len() && j + 1 < plans[i].len() {
                    let label2 = result[j + 1];
                    let door2 = plans[i][j + 1..j + 2].parse::<usize>().unwrap();
                    let next_label2 = result[j + 2];
                    label_observation.add_child_2step(label, door, label2, door2, next_label2);
                }

                // 3ステップの観測
                if j + 3 < result.len() && j + 2 < plans[i].len() {
                    let label2 = result[j + 1];
                    let door2 = plans[i][j + 1..j + 2].parse::<usize>().unwrap();
                    let label3 = result[j + 2];
                    let door3 = plans[i][j + 2..j + 3].parse::<usize>().unwrap();
                    let next_label3 = result[j + 3];
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
        for label0 in 0..4 {
            for door0 in 0..6 {
                for label1 in 0..4 {
                    if label_observation.is_unique_1step(label0 as i8, door0, label1 as i8) {
                        println!(
                            "Label {} + Door {} : Label {} is unique",
                            label0, door0, label1
                        );
                    } else {
                        for door1 in 0..6 {
                            for label2 in 0..4 {
                                if label_observation.is_unique_2step(
                                    label0 as i8,
                                    door0,
                                    label1 as i8,
                                    door1,
                                    label2 as i8,
                                ) {
                                    println!(
                                        "Label {} + Door {} + Label {} + Door {} + Label {} is unique",
                                        label0, door0, label1, door1, label2
                                    );
                                } else {
                                    for door2 in 0..6 {
                                        for label3 in 0..4 {
                                            if label_observation.is_unique_3step(
                                                &[label0 as i8, label1 as i8, label2 as i8],
                                                &[door0, door1, door2],
                                                label3 as i8,
                                            ) {
                                                println!(
                                                    "Label {} + Door {} + Label {} + Door {} + Label {} + Door {}: Label {} is unique",
                                                    label0,
                                                    door0,
                                                    label1,
                                                    door1,
                                                    label2,
                                                    door2,
                                                    label3
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(label_observation)
    }

    fn guess(
        &self,
        node_count: usize,
        plans: &[String],
        results: &[Vec<i8>],
        label_observation: &LabelObservation,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let mut num_rooms = [0; 4];

        for (i, num_room) in num_rooms.iter_mut().enumerate() {
            *num_room = label_observation.get_size(i as i8);
        }

        let sum_num_rooms = label_observation.get_sum_size();

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
                let door_i2 = if i + 1 < plan.len() {
                    plans[k][i + 1..i + 2].parse::<usize>().unwrap()
                } else {
                    !0
                };
                let label_i2 = if i + 1 < results[k].len() {
                    results[k][i + 1]
                } else {
                    !0
                };
                let door_i3 = if i + 2 < plan.len() {
                    plans[k][i + 2..i + 3].parse::<usize>().unwrap()
                } else {
                    !0
                };
                let label_i3 = if i + 2 < results[k].len() {
                    results[k][i + 2]
                } else {
                    !0
                };
                let label_i4 = if i + 3 < results[k].len() {
                    results[k][i + 3]
                } else {
                    !0
                };
                let mut check_step = 0;
                if label_observation.is_unique_1step(label_i, door_i, label_i2) {
                    check_step = 1;
                } else if label_i2 != !0
                    && label_observation
                        .is_unique_2step(label_i, door_i, label_i2, door_i2, label_i3)
                {
                    check_step = 2;
                } else if label_i3 != !0
                    && label_observation.is_unique_3step(
                        &[label_i, label_i2, label_i3],
                        &[door_i, door_i2, door_i3],
                        label_i4,
                    )
                {
                    check_step = 3;
                } else if num_rooms[label_i as usize] > 1 {
                    continue;
                }
                for j in i + 1..plans[k].len() - 1 {
                    let label_j = results[k][j];
                    if label_i != label_j {
                        continue;
                    }
                    if num_rooms[label_i as usize] > 1 {
                        let door_j = plans[k][j..j + 1].parse::<usize>().unwrap();
                        let label_j2 = results[k][j + 1];

                        // 選んだドアとその行先が同じなら同じ部屋とみなす
                        if door_i != door_j {
                            continue;
                        }
                        if label_i2 != label_j2 {
                            continue;
                        }
                        if check_step >= 2 {
                            let door_j2 = if j + 1 < plans[k].len() {
                                plans[k][j + 1..j + 2].parse::<usize>().unwrap()
                            } else {
                                !0
                            };
                            let label_j3 = if j + 2 < results[k].len() {
                                results[k][j + 2]
                            } else {
                                !0
                            };
                            if door_i2 != door_j2 {
                                continue;
                            }
                            if label_i3 != label_j3 {
                                continue;
                            }
                            if check_step >= 3 {
                                let door_j3 = if j + 2 < plans[k].len() {
                                    plans[k][j + 2..j + 3].parse::<usize>().unwrap()
                                } else {
                                    !0
                                };
                                let label_j4 = if j + 3 < results[k].len() {
                                    results[k][j + 3]
                                } else {
                                    !0
                                };
                                if door_i3 != door_j3 {
                                    continue;
                                }
                                if label_i4 != label_j4 {
                                    continue;
                                }
                            }
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
        println!("Start index: {}", uf.find(0));

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

        let guess_resp = self.requester.guess(GuessRequestMap {
            rooms: node_label.clone(),
            starting_room: start_index,
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
    let team_id = args.get(2).cloned().unwrap_or_else(|| "hoge".to_string());

    let requester = Requester::new(args.get(2).cloned());

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
