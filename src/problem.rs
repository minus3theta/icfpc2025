use std::collections::HashSet;

use rand::Rng;

pub struct Problem {
    starting_room: usize,
    connections: Vec<Vec<usize>>,
    max_plan_length: usize,
}

const DOOR_COUNT: usize = 6;

impl Problem {
    pub fn new(size: usize, max_plan_length: usize, ploidy: usize, seed: Option<u64>) -> Self {
        let size = size / ploidy;
        // 各部屋は 6　つの扉を持つ
        let mut remain_count = vec![DOOR_COUNT; size];
        let mut connections = vec![vec![None; DOOR_COUNT]; size];
        let mut available_doors = 6;
        let mut unconnected = HashSet::new();
        for i in 0..size {
            unconnected.insert(i);
        }
        let mut connected = HashSet::new();

        let mut random: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(
            seed.unwrap_or_else(|| rand::rng().random_range(0..u64::MAX)),
        );

        // 初期位置をランダムに選択する
        let mut starting_room = random.random_range(0..size);
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
                    remain_count[from_room] -= 1;
                    available_doors -= 1;
                    if from_room == to_room && from_door == to_door {
                        // 自己ループでは 1 つだけ減らす
                        continue;
                    }
                    connections[to_room][to_door] = Some(from_room);
                    remain_count[to_room] -= 1;
                    available_doors -= 1;
                }
            }
        }

        assert!(available_doors == 0);

        if ploidy > 1 {
            // 乱数付きで n 倍にする
            starting_room += random.random_range(0..ploidy) * size;
            connections = itertools::repeat_n(connections, ploidy)
                .enumerate()
                .flat_map(|(i, v)| {
                    v.into_iter()
                        .map(|v| {
                            v.into_iter()
                                .map(|v| Some(v.unwrap() + i * size))
                                .collect::<Vec<Option<usize>>>()
                        })
                        .collect::<Vec<_>>()
                })
                .collect();

            // ランダムに ploidy 間で接続を入れ替える
            for from_room in 0..size {
                for from_door in 0..DOOR_COUNT {
                    let to_room = connections[from_room][from_door].unwrap();
                    if to_room % size < from_room % size {
                        continue;
                    }
                    for to_door in 0..DOOR_COUNT {
                        if from_room == to_room && from_door <= to_door {
                            continue;
                        }
                        if connections[to_room][to_door].unwrap() == from_room {
                            let delta = random.random_range(0..ploidy) * size;
                            for i in 0..ploidy {
                                connections[from_room + i * size][from_door] = connections
                                    [from_room + i * size][from_door]
                                    .map(|v| (v + delta) % (size * ploidy));
                                connections[to_room + i * size][to_door] = connections
                                    [to_room + i * size][to_door]
                                    .map(|v| (v + size * ploidy - delta) % (size * ploidy));
                            }
                            break;
                        }
                    }
                }
            }
        }

        Self {
            starting_room,
            connections: connections
                .into_iter()
                .map(|v| v.into_iter().map(|v| v.unwrap()).collect())
                .collect(),
            max_plan_length,
        }
    }

    pub fn explore(&self, plan: &str) -> Result<Vec<usize>, String> {
        let mut room_digits = (0..self.connections.len()).collect::<Vec<usize>>();

        let mut result = Vec::new();
        let mut current_room = self.starting_room;

        result.push(room_digits[current_room]);

        let mut remaining_plan = self.max_plan_length;
        let mut writing_mode = false;
        for c in plan.chars() {
            if c == '[' {
                writing_mode = true;
                continue;
            }
            if c == ']' {
                writing_mode = false;
                continue;
            }
            if writing_mode {
                if !('0'..'4').contains(&c) {
                    return Err(format!("Invalid plan: {}", plan));
                }
                room_digits[current_room] = (c as u8 - b'0') as usize;
                result.push(room_digits[current_room]);
            } else {
                if !('0'..'6').contains(&c) {
                    return Err(format!("Invalid plan: {}", plan));
                }
                current_room = self.connections[current_room][(c as u8 - b'0') as usize];
                result.push(room_digits[current_room]);

                if remaining_plan == 0 {
                    return Err(format!("Plan length is too long: {}", plan.len()));
                }
                remaining_plan -= 1;
            }
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
        if current_room >= connections.len() {
            return;
        }
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
        if rooms.len() != self.connections.len() {
            return Ok(false);
        }

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
