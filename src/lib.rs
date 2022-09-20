use reqwest::{ header, Response, Error };
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
    #[error("Failed to create HTTP client: {0}")]
    ClientBuilder(String),
    #[error("{0}")]
    ResponseBodyToBig(String),
    #[error("Unknown error")]
    Unknown,
}

// For auto-converting reqwest errors to our error type
impl From<reqwest::Error> for DeSecError {
    fn from(error: reqwest::Error) -> Self {
        DeSecError::Request(error.to_string())
    }
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
pub struct Domain {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keys: Option<Vec<DNSSECKeyInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_ttl: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub touched: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zonefile: Option<String>
}

type DomainList = Vec<Domain>;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DNSSECKeyInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dnskey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ds: Option<Vec<String>>,
    #[serde(rename = "flags")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyflags: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keytype: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed: Option<bool>,
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
    client: reqwest::Client,
    pub api_url: String,
    pub token: String
}

// Used by all request methods (get, post, patch, delete) in order
// to evaluate the result of the request on convert types in case of errors
/*
fn eval_ureq_result(result: Result<Response, Error>) -> Result<Response, DeSecError> {

    match result {
        Ok(response) => {
            // If the response is larger than 10 megabytes, into_string() will return an error.
            // https://docs.rs/ureq/2.5.0/ureq/struct.Response.html#method.into_string
            // In our usecase the responses should never get that big.
            Ok(response)
        },
        Err(error) => {
            Err(DeSecError::Request(error.to_string()))
        }
        /*
        Err(ureq::Error::Status(404, response)) => {
            let status_text = response.status_text().to_string();
            let body = response.into_string().map_err(|err| DeSecError::ResponseBodyToBig(err.to_string()))?;
            Err(DeSecError::RessourceNotFound(format!("{},{}", status_text, body)))
        },
        Err(ureq::Error::Status(code, response)) => {
            let status_text = response.status_text().to_string();
            let body = response.into_string().map_err(|err| DeSecError::ResponseBodyToBig(err.to_string()))?;
            Err(DeSecError::Request(format!("{},{},{}", code, status_text, body)))
        },
        Err(ureq::Error::Transport(transport)) => {
            Err(DeSecError::Transport(transport.to_string()))
        }
        */
    }
}
*/

impl DeSecClient {

    pub fn new(token: String) -> Result<Self, DeSecError> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(
                format!("Token {}", token.as_str()).as_str()
            ).unwrap()
        );
        let client = reqwest::ClientBuilder::new()
            .user_agent("rust-desec-client")
            .default_headers(headers)
            .build()
            .map_err(|error| DeSecError::ClientBuilder(error.to_string()))?;
        Ok(DeSecClient { client, api_url: API_URL.into(), token })
    }

    pub async fn get_account_info(&self) -> Result<AccountInformation, DeSecError> {
        self.get("/auth/account/")
            .await?
            .json()
            .await
            .map_err(DeSecError::from)
    }

    pub async fn create_domain(&self, domain: String) -> Result<Domain, DeSecError> {
        self.post(
            "/domains/",
            format!("{{\"name\": \"{}\"}}", domain)
        )
        .await?
        .json()
        .await
        .map_err(DeSecError::from)
    }

    pub async fn get_domains(&self) -> Result<DomainList, DeSecError> {
        self.get("/domains/")
            .await?
            .json()
            .await
            .map_err(DeSecError::from)
    }

    pub async fn get_domain(&self, domain: &str) -> Result<Domain, DeSecError> {
        self.get(format!("/domains/{}/", domain).as_str())
            .await?
            .json()
            .await
            .map_err(DeSecError::from)
    }

    pub async fn delete_domain(&self, domain: &str) -> Result<String, DeSecError> {
        self.delete(format!("/domains/{}/", domain).as_str())
            .await?
            .text()
            .await
            .map_err(DeSecError::from)
    }

    pub async fn get_zonefile(&self, domain: &str) -> Result<String, DeSecError> {
        self.get(format!(
            "/domains/{}/zonefile/",
            domain
        ).as_str())
        .await?
        .text()
        .await
        .map_err(DeSecError::from)
    }

    pub async fn create_rrset(&self, domain: String, subname: String, rrset_type: String, records: Vec<String>, ttl: u64) -> Result<ResourceRecordSet, DeSecError> {
        let rrset = ResourceRecordSet {
            domain: Some(domain.clone()),
            subname: Some(subname),
            rrset_type: Some(rrset_type),
            records: Some(records),
            ttl: Some(ttl),
            ..ResourceRecordSet::default()
        };
        self.post(
            format!("/domains/{}/rrsets/", domain).as_str(),
            serde_json::to_string(&rrset).map_err(|err| DeSecError::Parser(err.to_string()))?
        )
        .await?
        .json()
        .await
        .map_err(DeSecError::from)
    }

    pub async fn get_rrsets(&self, domain: &str) -> Result<ResourceRecordSetList, DeSecError> {
        self.get(format!("/domains/{}/rrsets/", domain).as_str())
        .await?
        .json()
        .await
        .map_err(DeSecError::from)
    }

    pub async fn get_rrset(&self, domain: &str, subname: &str, rrset_type: &str) -> Result<ResourceRecordSet, DeSecError> {
        self.get(format!(
            "/domains/{}/rrsets/{}/{}/",
            domain, subname, rrset_type
        ).as_str())
        .await?
        .json()
        .await
        .map_err(DeSecError::from)
    }

    pub async fn update_rrset(&self, domain: &str, subname: &str, rrset_type: &str, patch: &ResourceRecordSet) -> Result<ResourceRecordSet, DeSecError> {
        self.patch(
            format!(
                "/domains/{}/rrsets/{}/{}/"
                , domain, subname, rrset_type).as_str(),
            serde_json::to_string(patch).map_err(|err| DeSecError::Parser(err.to_string()))?
        )
        .await?
        .json()
        .await
        .map_err(DeSecError::from)
    }

    pub async fn delete_rrset(&self, domain: &str, subname: &str, rrset_type: &str) -> Result<String, DeSecError> {
        self.delete(
            format!(
                "/domains/{}/rrsets/{}/{}/"
                , domain, subname, rrset_type
            ).as_str()
        )
        .await?
        .text()
        .await
        .map_err(DeSecError::from)
    }

    async fn get(&self, endpoint: &str) -> Result<Response, Error> {
        self.client.get(format!("{}{}", self.api_url, endpoint))
            .send()
            .await
    }

    async fn post(&self, endpoint: &str, body: String) -> Result<Response, Error> {
        self.client.post(format!("{}{}", self.api_url, endpoint).as_str())
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .await
    }

    async fn patch(&self, endpoint: &str, body: String) -> Result<Response, Error> {
        self.client.patch(format!("{}{}", self.api_url, endpoint).as_str())
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
    }

    async fn delete(&self, endpoint: &str) -> Result<Response, Error> {
        self.client.delete(format!("{}{}", self.api_url, endpoint).as_str())
            .send()
            .await
    }
}
