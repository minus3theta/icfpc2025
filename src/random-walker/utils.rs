use std::collections::HashMap;

pub struct Action {
    pub label: usize,
    pub door: usize,
}

impl Action {
    pub fn new(label: usize, door: usize) -> Self {
        Self { label, door }
    }
}

#[derive(Debug)]
pub struct UnionFind {
    parent: Vec<usize>,
    edge: Vec<Vec<usize>>,
    size: Vec<usize>,
}

impl UnionFind {
    pub fn new(n: usize) -> Self {
        Self {
            parent: vec![!0; n],
            edge: vec![vec![!0; 6]; n],
            size: vec![1; n],
        }
    }

    pub fn whole_size(&self) -> usize {
        self.parent.len()
    }

    pub fn set_edge(&mut self, src: usize, door: usize, dst: usize) {
        self.edge[src][door] = self.find(dst);
    }

    pub fn find(&mut self, x: usize) -> usize {
        if self.parent[x] == !0 {
            x
        } else {
            self.parent[x] = self.find(self.parent[x]);
            self.parent[x]
        }
    }

    pub fn get_size(&mut self, x: usize) -> usize {
        let x = self.find(x);
        self.size[x]
    }

    pub fn get_edges(&mut self, x: usize) -> &Vec<usize> {
        for i in 0..6 {
            if self.edge[x][i] != !0 {
                self.edge[x][i] = self.find(self.edge[x][i]);
            }
        }
        &self.edge[x]
    }

    pub fn union(&mut self, x: usize, y: usize) -> usize {
        let root_x = self.find(x);
        let root_y = self.find(y);
        if root_x == root_y {
            return 0;
        }
        // 一連の操作でマージされた頂点の組の数を返す
        let mut additional_union = Vec::new();
        self.parent[root_y] = root_x;
        self.size[root_x] += self.size[root_y];
        let mut merged_count = 1;
        for i in 0..6 {
            if self.edge[root_y][i] == !0 {
                continue;
            }
            if self.edge[root_x][i] == !0 {
                self.edge[root_x][i] = self.edge[root_y][i];
            } else {
                additional_union.push((self.edge[root_x][i], self.edge[root_y][i]));
            }
        }
        for (x, y) in additional_union {
            merged_count += self.union(x, y);
        }
        merged_count
    }

    pub fn get_next(&mut self, start: usize, door: usize) -> usize {
        if start != !0 && self.edge[start][door] != !0 {
            self.find(self.edge[start][door])
        } else {
            !0
        }
    }

    pub fn get_path(&mut self, start: usize, actions: &[Action]) -> Vec<usize> {
        let mut res = Vec::new();
        let mut cur = start;
        for Action { label: _, door } in actions {
            cur = self.get_next(cur, *door);
            res.push(cur);
        }
        res
    }
}

struct LabelObservationNode {
    child: HashMap<usize, HashMap<usize, LabelObservationNode>>,
}

impl LabelObservationNode {
    fn new() -> Self {
        Self {
            child: HashMap::new(),
        }
    }

    fn add_child(&mut self, door: usize, label: usize) {
        if !self.child.contains_key(&door) || !self.child[&door].contains_key(&label) {
            self.child
                .entry(door)
                .or_default()
                .insert(label, LabelObservationNode::new());
        }
    }

    fn get_child(&self, door: usize, label: usize) -> Option<&LabelObservationNode> {
        self.child.get(&door).and_then(|c| c.get(&label))
    }

    fn get_child_mut(&mut self, door: usize, label: usize) -> Option<&mut LabelObservationNode> {
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

pub struct LabelObservation {
    nodes: [LabelObservationNode; 4],
    room_count: [usize; 4],
}

impl LabelObservation {
    pub fn new(node_count: usize) -> Self {
        Self {
            nodes: [
                LabelObservationNode::new(),
                LabelObservationNode::new(),
                LabelObservationNode::new(),
                LabelObservationNode::new(),
            ],
            room_count: [
                node_count.div_ceil(4),
                (node_count + 2) / 4,
                (node_count + 1) / 4,
                node_count / 4,
            ],
        }
    }

    pub fn get_room_count(&self) -> &[usize; 4] {
        &self.room_count
    }

    pub fn get_size(&self, label: usize) -> usize {
        self.nodes[label].get_size()
    }

    pub fn get_sum_size(&self) -> usize {
        self.nodes.iter().map(|n| n.get_size()).sum()
    }

    pub fn add_child(&mut self, label: usize, door: usize, destination_label: usize) {
        self.nodes[label].add_child(door, destination_label);
    }

    pub fn add_child_2step(
        &mut self,
        label0: usize,
        door0: usize,
        label1: usize,
        door1: usize,
        destination_label: usize,
    ) {
        self.add_child(label0, door0, label1);
        self.nodes[label0]
            .get_child_mut(door0, label1)
            .unwrap()
            .add_child(door1, destination_label);
    }

    pub fn add_child_3step(&mut self, label: &[usize], door: &[usize], destination_label: usize) {
        self.add_child_2step(label[0], door[0], label[1], door[1], label[2]);
        self.nodes[label[0]]
            .get_child_mut(door[0], label[1])
            .unwrap()
            .get_child_mut(door[1], label[2])
            .unwrap()
            .add_child(door[2], destination_label);
    }

    pub fn is_unique_1step(&self, label: usize, door: usize, destination_label: usize) -> bool {
        // このドアを選んだ場合の行先が、このラベルに対応する部屋の中で最も多くの行先を持つドアであり、かつその行先が一意かどうかをチェック
        self.room_count[label] == self.get_size(label)
            && self.nodes[label].is_max_selection(door)
            && self.nodes[label]
                .get_child(door, destination_label)
                .is_some()
            && self.nodes[label]
                .get_child(door, destination_label)
                .unwrap()
                .is_unique()
    }

    pub fn is_unique_2step(
        &self,
        label0: usize,
        door0: usize,
        label1: usize,
        door1: usize,
        destination_label: usize,
    ) -> bool {
        // このドアを選んだ場合の行先が、このラベルに対応する部屋の中で最も多くの行先を持つドアであり、かつその行先が一意かどうかをチェック
        if self.room_count[label0] != self.get_size(label0) {
            return false;
        }
        if !self.nodes[label0].is_max_selection(door0) {
            return false;
        }
        let child = self.nodes[label0].get_child(door0, label1);
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

    pub fn is_unique_3step(
        &self,
        label: &[usize],
        door: &[usize],
        destination_label: usize,
    ) -> bool {
        // このドアを選んだ場合の行先が、このラベルに対応する部屋の中で最も多くの行先を持つドアであり、かつその行先が一意かどうかをチェック
        if self.room_count[label[0]] != self.get_size(label[0]) {
            return false;
        }
        if !self.nodes[label[0]].is_max_selection(door[0]) {
            return false;
        }
        let child0 = self.nodes[label[0]].get_child(door[0], label[1]);
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

    pub fn get_unique_paths(&self) -> Vec<(Vec<Action>, usize)> {
        // 3step以内で部屋の区別が可能なパスをすべて列挙
        let mut res = Vec::new();
        for label0 in 0..4 {
            for door0 in 0..6 {
                for label1 in 0..4 {
                    if self.is_unique_1step(label0, door0, label1) {
                        res.push((vec![Action::new(label0, door0)], label1));
                    } else {
                        for door1 in 0..6 {
                            for label2 in 0..4 {
                                if self.is_unique_2step(label0, door0, label1, door1, label2) {
                                    res.push((
                                        vec![
                                            Action::new(label0, door0),
                                            Action::new(label1, door1),
                                        ],
                                        label2,
                                    ));
                                } else {
                                    for door2 in 0..6 {
                                        for label3 in 0..4 {
                                            if self.is_unique_3step(
                                                &[label0, label1, label2],
                                                &[door0, door1, door2],
                                                label3,
                                            ) {
                                                res.push((
                                                    vec![
                                                        Action::new(label0, door0),
                                                        Action::new(label1, door1),
                                                        Action::new(label2, door2),
                                                    ],
                                                    label3,
                                                ));
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
        res
    }
}
