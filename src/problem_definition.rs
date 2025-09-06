use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

#[derive(Deserialize)]
struct ProblemDefinitionJson {
    name: String,
    size: usize,
    version: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProblemDefinition {
    pub name: String,
    pub size: usize,
    pub version: usize,
    pub max_plan_length: usize,
    pub label_rewritable: bool,
}

pub struct ProblemDefinitions {
    definitions: HashMap<String, ProblemDefinition>,
}

impl ProblemDefinitions {
    pub fn new() -> Self {
        let file = File::open("problems.json").unwrap();
        let reader = BufReader::new(file);
        let definitions = serde_json::from_reader::<_, Vec<ProblemDefinitionJson>>(reader).unwrap();
        Self {
            definitions: definitions
                .into_iter()
                .map(|d| {
                    (
                        d.name.clone(),
                        ProblemDefinition {
                            name: d.name,
                            size: d.size,
                            version: d.version,
                            max_plan_length: match d.version {
                                1 => 18 * d.size,
                                2 => 6 * d.size,
                                _ => 0,
                            },
                            label_rewritable: match d.version {
                                2 => true,
                                _ => false,
                            },
                        },
                    )
                })
                .collect(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&ProblemDefinition> {
        self.definitions.get(name)
    }
}
