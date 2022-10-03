use reqwest::{header, Error, Response, StatusCode};
use serde::{Deserialize, Serialize};

static API_URL: &str = "https://desec.io/api/v1";

#[derive(thiserror::Error, Debug)]
pub enum DeSecError {
    #[error("Operation cannot be performed: {0}")]
    BadRequest(String),
    #[error("An http error occured: {0}")]
    Http(#[from] Error),
    #[error("Bulk request rejected: {0}")]
    HttpBulk(serde_json::Value),
    #[error("An unknown http status code has been received")]
    HttpUnexpectedStatus(Response),
    #[error("The requet resource does not exist")]
    NotFound,
    #[error("You reached your deSEC domain limit")]
    DomainLimit,
    #[error("Failed to parse the response: {0}")]
    Parser(String),
    #[error("Failed to create HTTP client: {0}")]
    ClientBuilder(String),
    #[error("Failed to create HTTP client: {0}")]
    Generic(String),
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
    pub zonefile: Option<String>,
}

pub type DomainList = Vec<Domain>;

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
    pub touched: Option<String>,
}

pub type ResourceRecordSetList = Vec<ResourceRecordSet>;

pub struct DeSecClient {
    client: reqwest::Client,
    api_url: String,
}

impl DeSecClient {
    pub fn new(token: String) -> Result<Self, DeSecError> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(format!("Token {}", token.as_str()).as_str()).unwrap(),
        );
        let client = reqwest::ClientBuilder::new()
            .user_agent("rust-desec-client")
            .default_headers(headers)
            .build()
            .map_err(|error| DeSecError::ClientBuilder(error.to_string()))?;
        Ok(DeSecClient {
            client,
            api_url: API_URL.into(),
        })
    }

    pub async fn create_domain(&self, domain: &str) -> Result<Domain, DeSecError> {
        match self
            .post("/domains/", format!("{{\"name\": \"{}\"}}", domain))
            .await
        {
            Ok(response) => match response.status() {
                StatusCode::CREATED => response.json().await.map_err(|error| error.into()),
                StatusCode::BAD_REQUEST => Err(DeSecError::BadRequest(response.text().await?)),
                StatusCode::FORBIDDEN => Err(DeSecError::DomainLimit),
                _ => Err(DeSecError::HttpUnexpectedStatus(response)),
            },
            Err(error) => Err(error.into()),
        }
    }

    pub async fn get_domains(&self) -> Result<DomainList, DeSecError> {
        match self.get("/domains/").await {
            Ok(response) => match response.status() {
                StatusCode::CREATED => response.json().await.map_err(|error| error.into()),
                _ => Err(DeSecError::HttpUnexpectedStatus(response)),
            },
            Err(error) => Err(error.into()),
        }
    }

    pub async fn get_domain(&self, domain: &str) -> Result<Domain, DeSecError> {
        match self.get(format!("/domains/{}/", domain).as_str()).await {
            Ok(response) => match response.status() {
                StatusCode::CREATED => response.json().await.map_err(|error| error.into()),
                StatusCode::NOT_FOUND => Err(DeSecError::NotFound),
                _ => Err(DeSecError::HttpUnexpectedStatus(response)),
            },
            Err(error) => Err(error.into()),
        }
    }

    pub async fn delete_domain(&self, domain: &str) -> Result<(), DeSecError> {
        match self.delete(format!("/domains/{}/", domain).as_str()).await {
            Ok(response) => match response.status() {
                StatusCode::NO_CONTENT => Ok(()),
                _ => Err(DeSecError::HttpUnexpectedStatus(response)),
            },
            Err(error) => Err(error.into()),
        }
    }

    pub async fn get_zonefile(&self, domain: &str) -> Result<String, DeSecError> {
        match self
            .get(format!("/domains/{}/zonefile/", domain).as_str())
            .await
        {
            Ok(response) => match response.status() {
                StatusCode::OK => response.text().await.map_err(|error| error.into()),
                StatusCode::NOT_FOUND => Err(DeSecError::NotFound),
                _ => Err(DeSecError::HttpUnexpectedStatus(response)),
            },
            Err(error) => Err(error.into()),
        }
    }

    pub async fn create_rrset(
        &self,
        domain: String,
        subname: String,
        rrset_type: String,
        records: Vec<String>,
        ttl: u64,
    ) -> Result<ResourceRecordSet, DeSecError> {
        let rrset = ResourceRecordSet {
            domain: Some(domain.clone()),
            subname: Some(subname),
            rrset_type: Some(rrset_type),
            records: Some(records),
            ttl: Some(ttl),
            ..ResourceRecordSet::default()
        };
        match self
            .post(
                format!("/domains/{}/rrsets/", domain).as_str(),
                serde_json::to_string(&rrset).map_err(|err| DeSecError::Parser(err.to_string()))?,
            )
            .await
        {
            Ok(response) => match response.status() {
                StatusCode::CREATED => response.json().await.map_err(|error| error.into()),
                StatusCode::BAD_REQUEST => Err(DeSecError::BadRequest(response.text().await?)),
                _ => Err(DeSecError::HttpUnexpectedStatus(response)),
            },
            Err(error) => Err(error.into()),
        }
    }

    pub async fn create_rrset_bulk(
        &self,
        domain: String,
        rrsets: ResourceRecordSetList,
    ) -> Result<(), DeSecError> {
        match self
            .post(
                format!("/domains/{}/rrsets/", domain).as_str(),
                serde_json::to_string(&rrsets)
                    .map_err(|err| DeSecError::Parser(err.to_string()))?,
            )
            .await
        {
            Ok(response) => match response.status() {
                StatusCode::CREATED => Ok(()),
                StatusCode::BAD_REQUEST => Err(DeSecError::HttpBulk(response.json().await?)),
                _ => Err(DeSecError::HttpUnexpectedStatus(response)),
            },
            Err(error) => Err(error.into()),
        }
    }

    pub async fn get_rrsets(&self, domain: &str) -> Result<ResourceRecordSetList, DeSecError> {
        match self
            .get(format!("/domains/{}/rrsets/", domain).as_str())
            .await
        {
            Ok(response) => match response.status() {
                StatusCode::OK => response.json().await.map_err(|error| error.into()),
                StatusCode::NOT_FOUND => Err(DeSecError::NotFound),
                _ => Err(DeSecError::HttpUnexpectedStatus(response)),
            },
            Err(error) => Err(error.into()),
        }
    }

    pub async fn get_rrset(
        &self,
        domain: &str,
        subname: &str,
        rrset_type: &str,
    ) -> Result<ResourceRecordSet, DeSecError> {
        match self
            .get(format!("/domains/{}/rrsets/{}/{}/", domain, subname, rrset_type).as_str())
            .await
        {
            Ok(response) => match response.status() {
                StatusCode::OK => response.json().await.map_err(|error| error.into()),
                StatusCode::NOT_FOUND => Err(DeSecError::NotFound),
                _ => Err(DeSecError::HttpUnexpectedStatus(response)),
            },
            Err(error) => Err(error.into()),
        }
    }

    pub async fn update_rrset(
        &self,
        domain: &str,
        subname: &str,
        rrset_type: &str,
        patch: &ResourceRecordSet,
    ) -> Result<ResourceRecordSet, DeSecError> {
        match self
            .patch(
                format!("/domains/{}/rrsets/{}/{}/", domain, subname, rrset_type).as_str(),
                serde_json::to_string(patch).map_err(|err| DeSecError::Parser(err.to_string()))?,
            )
            .await
        {
            Ok(response) => match response.status() {
                StatusCode::OK => response.json().await.map_err(|error| error.into()),
                StatusCode::NO_CONTENT => Ok(ResourceRecordSet::default()),
                StatusCode::BAD_REQUEST => Err(DeSecError::HttpBulk(response.json().await?)),
                _ => Err(DeSecError::HttpUnexpectedStatus(response)),
            },
            Err(error) => Err(error.into()),
        }
    }

    pub async fn update_rrset_bulk(
        &self,
        domain: String,
        rrsets: ResourceRecordSetList,
    ) -> Result<(), DeSecError> {
        match self
            .patch(
                format!("/domains/{}/rrsets/", domain).as_str(),
                serde_json::to_string(&rrsets)
                    .map_err(|err| DeSecError::Parser(err.to_string()))?,
            )
            .await
        {
            Ok(response) => match response.status() {
                StatusCode::CREATED => Ok(()),
                StatusCode::BAD_REQUEST => Err(DeSecError::HttpBulk(response.json().await?)),
                _ => Err(DeSecError::HttpUnexpectedStatus(response)),
            },
            Err(error) => Err(error.into()),
        }
    }

    pub async fn delete_rrset(
        &self,
        domain: &str,
        subname: &str,
        rrset_type: &str,
    ) -> Result<(), DeSecError> {
        match self
            .delete(format!("/domains/{}/rrsets/{}/{}/", domain, subname, rrset_type).as_str())
            .await
        {
            Ok(response) => match response.status() {
                StatusCode::NO_CONTENT => Ok(()),
                _ => Err(DeSecError::HttpUnexpectedStatus(response)),
            },
            Err(error) => Err(error.into()),
        }
    }

    async fn get(&self, endpoint: &str) -> Result<Response, Error> {
        self.client
            .get(format!("{}{}", self.api_url, endpoint))
            .send()
            .await
    }

    async fn post(&self, endpoint: &str, body: String) -> Result<Response, Error> {
        self.client
            .post(format!("{}{}", self.api_url, endpoint).as_str())
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .await
    }

    async fn patch(&self, endpoint: &str, body: String) -> Result<Response, Error> {
        self.client
            .patch(format!("{}{}", self.api_url, endpoint).as_str())
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
    }

    async fn delete(&self, endpoint: &str) -> Result<Response, Error> {
        self.client
            .delete(format!("{}{}", self.api_url, endpoint).as_str())
            .send()
            .await
    }
}
