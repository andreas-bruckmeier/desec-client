use serde::{Deserialize, Serialize};
use thiserror::Error;

static API_URL: &str = "https://desec.io/api/v1";

#[derive(Error, Debug)]
pub enum DeSecError {
    #[error("Transport error: {0}")]
    Transport(String),
    #[error("The request failed: {0}")]
    Request(String),
    #[error("Failed parsing the response: {0}")]
    Parser(serde_json::Error),
    #[error("Unknown error")]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug)]
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

pub struct DeSecClient {
    pub api_url: String,
    pub token: Option<String>
}

impl Default for DeSecClient {
    fn default() -> Self {
        DeSecClient {
            api_url: API_URL.to_string(),
            token: None
        }
    }
}

impl DeSecClient {

    pub fn new(token: String) -> Self {
        DeSecClient { token: Some(token), ..Default::default() }
    }

    pub fn get_account_info(self) -> Result<String, DeSecError> {
        self.get("/auth/account/".to_string())
    }

    pub fn get_rrset(self, domain: String, subname: String, rrset_type: String) -> Result<ResourceRecordSet, DeSecError> {
        serde_json::from_str(
            self.get(format!(
                "/domains/{}/rrsets/{}/{}/",
                domain, subname, rrset_type
            ))?.as_str()
        ).map_err(DeSecError::Parser)
    }

    pub fn get_rrsets(self, domain: String) -> Result<Vec<ResourceRecordSet>, DeSecError> {
        serde_json::from_str(
            self.get(format!("/domains/{}/rrsets/", domain))?.as_str()
        ).map_err(DeSecError::Parser)
    }

    pub fn create_apex_rrset(self, domain: String, rrset_type: String, records: Vec<String>, ttl: u64) -> Result<ResourceRecordSet, DeSecError> {
        self.create_rrset(domain, None, rrset_type, records, ttl)
    }

    pub fn create_subname_rrset(self, domain: String, subname: String, rrset_type: String, records: Vec<String>, ttl: u64) -> Result<ResourceRecordSet, DeSecError> {
        self.create_rrset(domain, Some(subname), rrset_type, records, ttl)
    }

    fn create_rrset(self, domain: String, subname: Option<String>, rrset_type: String, records: Vec<String>, ttl: u64) -> Result<ResourceRecordSet, DeSecError> {
        let rrset = ResourceRecordSet {
            created: None,
            domain: Some(domain.clone()),
            subname,
            name: None,
            rrset_type: Some(rrset_type),
            records: Some(records),
            ttl: Some(ttl),
            touched: None
        };
        serde_json::from_str(
            self.post(
                format!("/domains/{}/rrsets/", domain),
                serde_json::to_string(&rrset).map_err(DeSecError::Parser)?.as_str()
            )?.as_str()
        ).map_err(DeSecError::Parser)
    }

    pub fn update_apex_rrset(self, domain: String, rrset_type: String, patch: ResourceRecordSet) -> Result<ResourceRecordSet, DeSecError> {
        self.update_rrset(domain, None, rrset_type, patch)
    }

    pub fn update_subname_rrset(self, domain: String, subname: String, rrset_type: String, patch: ResourceRecordSet) -> Result<ResourceRecordSet, DeSecError> {
        self.update_rrset(domain, Some(subname), rrset_type, patch)
    }

    pub fn update_rrset(self, domain: String, subname: Option<String>, rrset_type: String, patch: ResourceRecordSet) -> Result<ResourceRecordSet, DeSecError> {
        serde_json::from_str(
            self.patch(
                format!(
                    "/domains/{}/rrsets/{}/{}/"
                    , domain, subname.clone().unwrap_or("@".to_string()), rrset_type),
                serde_json::to_string(&patch).map_err(DeSecError::Parser)?.as_str()
            )?.as_str()
        ).map_err(DeSecError::Parser)
    }

    fn get(self, endpoint: String) -> Result<String, DeSecError> {
        match ureq::get(format!("{}{}", self.api_url, endpoint).as_str())
        .set("Authorization", format!("Token {}", self.token.unwrap()).as_str())
        .call() {
            Ok(response) => {
                // If the response is larger than 10 megabytes, this will return an error.
                // https://docs.rs/ureq/2.5.0/ureq/struct.Response.html#method.into_string
                Ok(response.into_string().unwrap())
            },
            Err(ureq::Error::Status(_code, response)) => {
                Err(DeSecError::Request(response.status_text().to_string()))
            },
            Err(ureq::Error::Transport(transport)) => {
                Err(DeSecError::Transport(transport.message().unwrap_or("FOOO").to_string()))
            }
        }
    }

    fn post(self, endpoint: String, body: &str) -> Result<String, DeSecError> {
        match ureq::post(format!("{}{}", self.api_url, endpoint).as_str())
        .set("Authorization", format!("Token {}", self.token.unwrap()).as_str())
        .set("Content-Type", "application/json")
        .send_string(body) {
            Ok(response) => {
                // If the response is larger than 10 megabytes, this will return an error.
                // https://docs.rs/ureq/2.5.0/ureq/struct.Response.html#method.into_string
                Ok(response.into_string().unwrap())
            },
            Err(ureq::Error::Status(code, response)) => {
                let status_text = response.status_text().to_string();
                let body = response.into_string().unwrap_or("Response contains no body".to_string());
                Err(DeSecError::Request(format!("{},{},{}", code, status_text, body)))
            },
            Err(ureq::Error::Transport(transport)) => {
                Err(DeSecError::Transport(transport.message().unwrap_or("Error contains to message").to_string()))
            }
        }
    }

    fn patch(self, endpoint: String, body: &str) -> Result<String, DeSecError> {
        match ureq::patch(format!("{}{}", self.api_url, endpoint).as_str())
        .set("Authorization", format!("Token {}", self.token.unwrap()).as_str())
        .set("Content-Type", "application/json")
        .send_string(body) {
            Ok(response) => {
                // If the response is larger than 10 megabytes, this will return an error.
                // https://docs.rs/ureq/2.5.0/ureq/struct.Response.html#method.into_string
                Ok(response.into_string().unwrap())
            },
            Err(ureq::Error::Status(code, response)) => {
                let status_text = response.status_text().to_string();
                let body = response.into_string().unwrap_or("Response contains no body".to_string());
                Err(DeSecError::Request(format!("{},{},{}", code, status_text, body)))
            },
            Err(ureq::Error::Transport(transport)) => {
                Err(DeSecError::Transport(transport.message().unwrap_or("Error contains to message").to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DeSecClient;
    use super::ResourceRecordSet;
    use super::API_URL;

    fn apikey() -> Option<String> {
        match::std::env::var("DESEC_APIKEY") {
            Ok(key) => Some(key),
            Err(_r) => {
                None
            }
        }
    }

    fn domain() -> Option<String> {
        match::std::env::var("DESEC_DOMAIN") {
            Ok(domain) => Some(domain),
            Err(_r) => {
                None
            }
        }
    }

    #[test]
    fn test_default() {
        let client = DeSecClient::default();
        assert_eq!(client.api_url, API_URL);
    }

    #[test]
    fn test_accont_info() {
        match apikey() {
            Some(key) => {
               let client = DeSecClient::new(key.clone());
               let account_info = client.get_account_info();
               assert!(account_info.is_ok());
            },
            _ => {}
        }
    }

    #[test]
    fn test_get_rrset() {
        match apikey() {
            Some(key) => {
                match domain() {
                    Some(domain) => {
                        let client = DeSecClient::new(key.clone());
                        let rrsets = client.get_rrset(
                            domain,
                            "nginx.bruckmeier".to_string(),
                            "A".to_string());
                        assert!(rrsets.is_ok());
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }

    #[test]
    fn test_get_rrsets() {
        match apikey() {
            Some(key) => {
                match domain() {
                    Some(domain) => {
                        let client = DeSecClient::new(key.clone());
                        let rrsets = client.get_rrsets(domain);
                        assert!(rrsets.is_ok());
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }

    #[test]
    fn test_create_rrset() {
        match apikey() {
            Some(key) => {
                match domain() {
                    Some(domain) => {
                        let client = DeSecClient::new(key.clone());
                        let rrset = client.create_subname_rrset(
                            domain,
                            "mysubdomain".to_string(),
                            "A".to_string(),
                            vec!("8.8.8.8".to_string()),
                            3600);
                        assert!(rrset.is_ok());
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }

    #[test]
    fn test_patch_rrset() {
        match apikey() {
            Some(key) => {
                match domain() {
                    Some(domain) => {
                        let client = DeSecClient::new(key.clone());
                        let patch = ResourceRecordSet {
                            created: None,
                            domain: None,
                            subname: None,
                            name: None,
                            rrset_type: None,
                            records: Some(vec!("1.2.3.4".to_string())),
                            ttl: Some(5000),
                            touched: None
                        };
                        let rrset = client.update_subname_rrset(
                            domain,
                            "mysubdomain".to_string(),
                            "A".to_string(),
                            patch);
                        assert!(rrset.is_ok());
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }
}
