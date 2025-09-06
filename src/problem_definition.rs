use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProblemDefinition {
    pub name: String,
    pub size: usize,
}

pub struct ProblemDefinitions {
    definitions: HashMap<String, ProblemDefinition>,
}

impl ProblemDefinitions {
    pub fn new() -> Self {
        let file = File::open("problems.json").unwrap();
        let reader = BufReader::new(file);
        let definitions = serde_json::from_reader::<_, Vec<ProblemDefinition>>(reader).unwrap();
        Self {
            definitions: definitions
                .into_iter()
                .map(|d| (d.name.clone(), d))
                .collect(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&ProblemDefinition> {
        self.definitions.get(name)
    }
}
