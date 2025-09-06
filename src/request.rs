mod types;

pub use types::*;

pub struct Requester {
    id: String,
    base_url: String,
    client: reqwest::blocking::Client,
}

impl Requester {
    pub fn new(id: Option<String>) -> Self {
        let base_url = if id.is_some() {
            "https://31pwr5t6ij.execute-api.eu-west-2.amazonaws.com".to_string()
        } else {
            "http://localhost:8080".to_string()
        };
        Self {
            id: id.unwrap_or_else(|| "hoge".to_string()),
            base_url,
            client: reqwest::blocking::Client::new(),
        }
    }

    pub fn select(
        &self,
        problem_name: String,
    ) -> Result<SelectResponse, Box<dyn std::error::Error>> {
        let select_req = SelectRequest {
            id: self.id.clone(),
            problem_name,
        };
        let response = self
            .client
            .post(format!("{}/select", self.base_url))
            .json(&select_req)
            .send()?;

        Ok(response.json::<SelectResponse>()?)
    }

    pub fn explore(
        &self,
        plans: Vec<String>,
    ) -> Result<ExploreResponse, Box<dyn std::error::Error>> {
        let explore_req = ExploreRequest {
            id: self.id.clone(),
            plans,
        };

        let response = self
            .client
            .post(format!("{}/explore", self.base_url))
            .json(&explore_req)
            .send()?;

        Ok(response.json::<ExploreResponse>()?)
    }

    pub fn guess(&self, map: GuessRequestMap) -> Result<GuessResponse, Box<dyn std::error::Error>> {
        let guess_req = GuessRequest {
            id: self.id.clone(),
            map,
        };

        let response = self
            .client
            .post(format!("{}/guess", self.base_url))
            .json(&guess_req)
            .send()?;

        Ok(response.json::<GuessResponse>()?)
    }
}
