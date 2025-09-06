use std::collections::HashSet;

use rand::Rng;

pub struct Problem {
    starting_room: usize,
    connections: Vec<Vec<usize>>,
}

const DOOR_COUNT: usize = 6;

impl Problem {
    pub fn new(size: usize) -> Self {
        // 各部屋は 6　つの扉を持つ
        let mut remain_count = vec![DOOR_COUNT; size];
        let mut connections = vec![vec![None; DOOR_COUNT]; size];
        let mut available_doors = 6;
        let mut unconnected = HashSet::new();
        for i in 0..size {
            unconnected.insert(i);
        }
        let mut connected = HashSet::new();

        let mut random = rand::rng();
        // 初期位置をランダムに選択する
        let starting_room = random.random_range(0..size);
        connected.insert(starting_room);
        unconnected.remove(&starting_room);

        // 全域木を生成する
        while !unconnected.is_empty() {
            // 接続元を選択する
            let mut index = random.random_range(0..available_doors);
            let mut from_room = usize::MAX;
            let mut from_door = usize::MAX;
            for r in connected.iter() {
                if index < remain_count[*r] {
                    from_room = *r;
                    for d in 0..DOOR_COUNT {
                        if connections[from_room][d].is_none() {
                            if index == 0 {
                                from_door = d;
                                break;
                            }
                            index -= 1;
                        }
                    }
                    break;
                }
                index -= remain_count[*r];
            }
            assert!(from_room != usize::MAX);
            assert!(from_door != usize::MAX);

            // 接続先を選択する
            let index = random.random_range(0..unconnected.len());
            let to_room = *unconnected.iter().nth(index).unwrap();
            let to_door = random.random_range(0..DOOR_COUNT);

            connected.insert(to_room);
            unconnected.remove(&to_room);
            available_doors += DOOR_COUNT;

            // 接続する
            connections[from_room][from_door] = Some(to_room);
            connections[to_room][to_door] = Some(from_room);
            remain_count[from_room] -= 1;
            remain_count[to_room] -= 1;
            available_doors -= 2;
        }

        assert!(unconnected.is_empty());
        assert!(connected.len() == size);
        assert!(available_doors == DOOR_COUNT * size - 2 * size + 2);

        // 未使用の扉を埋めていく
        for from_room in 0..size {
            for from_door in 0..DOOR_COUNT {
                if connections[from_room][from_door].is_none() {
                    remain_count[from_room] -= 1;
                    connections[from_room][from_door] = Some(usize::MAX);
                    available_doors -= 1;

                    // 接続先を選択する
                    let mut index = random.random_range(0..available_doors);
                    let mut to_room = usize::MAX;
                    let mut to_door = usize::MAX;
                    for r in connected.iter() {
                        if index < remain_count[*r] {
                            to_room = *r;
                            for d in 0..DOOR_COUNT {
                                if connections[to_room][d].is_none() {
                                    if index == 0 {
                                        to_door = d;
                                        break;
                                    }
                                    index -= 1;
                                }
                            }
                            break;
                        }
                        index -= remain_count[*r];
                    }
                    assert!(to_room != usize::MAX);
                    assert!(to_door != usize::MAX);

                    // 接続する
                    connections[from_room][from_door] = Some(to_room);
                    connections[to_room][to_door] = Some(from_room);
                    remain_count[to_room] -= 1;
                    available_doors -= 1;
                }
            }
        }

        assert!(available_doors == 0);

        Self {
            starting_room,
            connections: connections
                .into_iter()
                .map(|v| v.into_iter().map(|v| v.unwrap()).collect())
                .collect(),
        }
    }

    pub fn explore(&self, plan: &str) -> Result<Vec<usize>, String> {
        let mut result = Vec::new();
        let mut current_room = self.starting_room;

        result.push(current_room);

        for c in plan.chars() {
            if !('0'..'6').contains(&c) {
                return Err(format!("Invalid plan: {}", plan));
            }
            current_room = self.connections[current_room][(c as u8 - b'0') as usize];
            result.push(current_room);
        }
        Ok(result)
    }

    fn guess_visit(
        &self,
        connections: &Vec<Vec<usize>>,
        current_room: usize,
        guessed_room: usize,
        mapping: &mut Vec<usize>,
    ) {
        if mapping[current_room] != usize::MAX {
            return;
        }
        mapping[current_room] = guessed_room;
        for i in 0..6 {
            self.guess_visit(
                connections,
                connections[current_room][i],
                self.connections[guessed_room][i],
                mapping,
            );
        }
    }

    pub fn guess(
        &self,
        rooms: Vec<i8>,
        starting_room: usize,
        connections: Vec<Vec<usize>>,
    ) -> Result<bool, String> {
        let mut mapping = vec![usize::MAX; rooms.len()];

        self.guess_visit(
            &connections,
            starting_room,
            self.starting_room,
            &mut mapping,
        );

        for (i, room) in rooms.iter().enumerate() {
            if mapping[i] % 4 != *room as usize {
                return Ok(false);
            }
        }

        for (i, connection) in connections.iter().enumerate() {
            for (j, c) in connection.iter().enumerate() {
                if mapping[*c] != self.connections[mapping[i]][j] {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    pub fn pretty_print(&self) -> String {
        let mut result = format!("Starting room: {}", self.starting_room);
        for (i, connection) in self.connections.iter().enumerate() {
            result += &format!("\n{}:", i);
            for c in connection.iter() {
                result += &format!(" {}", c);
            }
        }
        result
    }
}
