use crate::types::*;

pub type Error = Box<dyn std::error::Error>;

pub trait Requester {
    fn select(&self, problem_name: String) -> Result<SelectResponse, Error>;
    fn explore(&self, plans: Vec<String>) -> Result<ExploreResponse, Error>;
    fn guess(&self, map: GuessRequestMap) -> Result<GuessResponse, Error>;
}

pub struct HttpRequester {
    id: String,
    base_url: String,
    client: reqwest::blocking::Client,
}

impl HttpRequester {
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
}

impl Requester for HttpRequester {
    fn select(&self, problem_name: String) -> Result<SelectResponse, Error> {
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

    fn explore(&self, plans: Vec<String>) -> Result<ExploreResponse, Error> {
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

    fn guess(&self, map: GuessRequestMap) -> Result<GuessResponse, Error> {
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
