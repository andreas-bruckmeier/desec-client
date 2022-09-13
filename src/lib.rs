use serde::{Deserialize, Serialize};
use thiserror::Error;

static API_URL: &str = "https://desec.io/api/v1";

#[derive(Error, Debug, Clone)]
pub enum DeSecError {
    #[error("Transport error: {0}")]
    Transport(String),
    #[error("The request failed: {0}")]
    Request(String),
    #[error("The requet resource does not exist: {0}")]
    RessourceNotFound(String),
    #[error("Failed parsing the response: {0}")]
    Parser(String),
    #[error("Unknown error")]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountInformation {
    pub created: String,
    pub email: String,
    pub id: String,
    pub limit_domains: u64,
    pub outreach_preference: bool
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ResourceRecordSet {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rrset_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub records: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub touched: Option<String>
}

type ResourceRecordSetList = Vec<ResourceRecordSet>;

#[derive(Debug, Clone)]
pub struct DeSecClient {
    pub api_url: String,
    pub token: String
}

// Used by all request methods (get, post, patch, delete) in order
// to evaluate the result of the request on convert types in case of errors
fn eval_ureq_result(result: Result<ureq::Response, ureq::Error>) -> Result<String, DeSecError> {

    match result {
        Ok(response) => {
            // If the response is larger than 10 megabytes, this will return an error.
            // https://docs.rs/ureq/2.5.0/ureq/struct.Response.html#method.into_string
            Ok(response.into_string().unwrap())
        },
        Err(ureq::Error::Status(404, response)) => {
            let status_text = response.status_text().to_string();
            let body = response.into_string().unwrap_or_else(|_| "Response contains no body".to_string());
            Err(DeSecError::RessourceNotFound(format!("{},{}", status_text, body)))
        },
        Err(ureq::Error::Status(code, response)) => {
            let status_text = response.status_text().to_string();
            let body = response.into_string().unwrap_or_else(|_| "Response contains no body".to_string());
            Err(DeSecError::Request(format!("{},{},{}", code, status_text, body)))
        },
        Err(ureq::Error::Transport(transport)) => {
            Err(DeSecError::Transport(transport.message().unwrap_or("Error contains to message").to_string()))
        }
    }
}

impl DeSecClient {

    pub fn new(token: String) -> Self {
        DeSecClient {
            api_url: API_URL.into(),
            token
        }
    }

    pub fn get_account_info(&self) -> Result<AccountInformation, DeSecError> {
        serde_json::from_str(
            self.get("/auth/account/".to_string().as_str())?.as_str()
        ).map_err(|err| DeSecError::Parser(err.to_string()))
    }

    pub fn get_rrset(&self, domain: &str, subname: &str, rrset_type: &str) -> Result<ResourceRecordSet, DeSecError> {
        serde_json::from_str(
            self.get(format!(
                "/domains/{}/rrsets/{}/{}/",
                domain, subname, rrset_type
            ).as_str())?.as_str()
        ).map_err(|err| DeSecError::Parser(err.to_string()))
    }

    pub fn get_rrsets(&self, domain: &str) -> Result<ResourceRecordSetList, DeSecError> {
        serde_json::from_str(
            self.get(format!("/domains/{}/rrsets/", domain).as_str())?.as_str()
        ).map_err(|err| DeSecError::Parser(err.to_string()))
    }

    pub fn create_rrset(&self, domain: String, subname: String, rrset_type: String, records: Vec<String>, ttl: u64) -> Result<ResourceRecordSet, DeSecError> {
        let rrset = ResourceRecordSet {
            domain: Some(domain.clone()),
            subname: Some(subname),
            rrset_type: Some(rrset_type),
            records: Some(records),
            ttl: Some(ttl),
            ..ResourceRecordSet::default()
        };
        serde_json::from_str(
            self.post(
                format!("/domains/{}/rrsets/", domain).as_str(),
                serde_json::to_string(&rrset).map_err(|err| DeSecError::Parser(err.to_string()))?.as_str()
            )?.as_str()
        ).map_err(|err| DeSecError::Parser(err.to_string()))
    }

    pub fn update_rrset(&self, domain: &str, subname: &str, rrset_type: &str, patch: &ResourceRecordSet) -> Result<ResourceRecordSet, DeSecError> {
        serde_json::from_str(
            self.patch(
                format!(
                    "/domains/{}/rrsets/{}/{}/"
                    , domain, subname, rrset_type).as_str(),
                serde_json::to_string(patch).map_err(|err| DeSecError::Parser(err.to_string()))?.as_str()
            )?.as_str()
        ).map_err(|err| DeSecError::Parser(err.to_string()))
    }

    pub fn delete_rrset(&self, domain: &str, subname: &str, rrset_type: &str) -> Result<String, DeSecError> {
        self.delete(
            format!(
                "/domains/{}/rrsets/{}/{}/"
                , domain, subname, rrset_type
            ).as_str()
        )
    }

    fn get(&self, endpoint: &str) -> Result<String, DeSecError> {
        eval_ureq_result(
            ureq::get(format!("{}{}", self.api_url, endpoint).as_str())
            .set("Authorization", format!("Token {}", &self.token).as_str())
            .call()
        )
    }

    fn post(&self, endpoint: &str, body: &str) -> Result<String, DeSecError> {
        eval_ureq_result(
            ureq::post(format!("{}{}", self.api_url, endpoint).as_str())
            .set("Authorization", format!("Token {}", &self.token).as_str())
            .set("Content-Type", "application/json")
            .send_string(body)
        )
    }

    fn patch(&self, endpoint: &str, body: &str) -> Result<String, DeSecError> {
        eval_ureq_result(
            ureq::patch(format!("{}{}", self.api_url, endpoint).as_str())
            .set("Authorization", format!("Token {}", &self.token).as_str())
            .set("Content-Type", "application/json")
            .send_string(body)
        )
    }

    fn delete(&self, endpoint: &str) -> Result<String, DeSecError> {
        eval_ureq_result(
            ureq::delete(format!("{}{}", self.api_url, endpoint).as_str())
            .set("Authorization", format!("Token {}", &self.token).as_str())
            .call()
        )
    }
}
