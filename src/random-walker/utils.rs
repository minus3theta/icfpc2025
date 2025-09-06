use std::collections::HashMap;

#[derive(Debug)]
pub struct UnionFind {
    pub parent: Vec<usize>,
    pub size: Vec<usize>,
}

impl UnionFind {
    pub fn new(n: usize) -> Self {
        Self {
            parent: vec![!0; n],
            size: vec![1; n],
        }
    }

    pub fn find(&mut self, x: usize) -> usize {
        if self.parent[x] == !0 {
            x
        } else {
            self.parent[x] = self.find(self.parent[x]);
            self.parent[x]
        }
    }

    pub fn union(&mut self, x: usize, y: usize) {
        let root_x = self.find(x);
        let root_y = self.find(y);
        if root_x == root_y {
            return;
        }
        self.parent[root_y] = root_x;
        self.size[root_x] += self.size[root_y];
    }
}

pub struct LabelObservationNode {
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

pub struct LabelObservation {
    nodes: [LabelObservationNode; 4],
}

impl LabelObservation {
    pub fn new() -> Self {
        Self {
            nodes: [
                LabelObservationNode::new(),
                LabelObservationNode::new(),
                LabelObservationNode::new(),
                LabelObservationNode::new(),
            ],
        }
    }

    pub fn get_size(&self, label: i8) -> usize {
        self.nodes[label as usize].get_size()
    }

    pub fn get_sum_size(&self) -> usize {
        self.nodes.iter().map(|n| n.get_size()).sum()
    }

    pub fn add_child(&mut self, label: i8, door: usize, destination_label: i8) {
        self.nodes[label as usize].add_child(door, destination_label);
    }

    pub fn add_child_2step(
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

    pub fn add_child_3step(&mut self, label: &[i8], door: &[usize], destination_label: i8) {
        self.add_child_2step(label[0], door[0], label[1], door[1], label[2]);
        self.nodes[label[0] as usize]
            .get_child_mut(door[0], label[1])
            .unwrap()
            .get_child_mut(door[1], label[2])
            .unwrap()
            .add_child(door[2], destination_label);
    }

    pub fn is_unique_1step(&self, label: i8, door: usize, destination_label: i8) -> bool {
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

    pub fn is_unique_2step(
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

    pub fn is_unique_3step(&self, label: &[i8], door: &[usize], destination_label: i8) -> bool {
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
