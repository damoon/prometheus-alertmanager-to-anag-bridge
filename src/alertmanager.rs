use actix_web::client::{Client, JsonPayloadError, SendRequestError};
use chrono::{DateTime, Duration, Utc};
use http::StatusCode;
use std::ops::Add;
use thiserror::Error;

pub async fn alerts() -> Result<Vec<Alert>, Error> {
    let mut response = new_http_client()
        .get("http://alertmanager:9093/api/v2/alerts")
        .header("Content-Type", "application/json")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(Error::BadStatus(response.status()));
    }

    let alerts: Vec<Alert> = response.json().await?;

    Ok(alerts)
}

pub async fn ack(name: String, comment: String) -> Result<(), Error> {
    let mut matchers = Vec::<Matcher>::new();
    matchers.push(Matcher {
        name: "alertname".to_string(),
        value: name,
        is_regex: false,
    });

    let starts_at: DateTime<Utc> = Utc::now();
    let ends_at = starts_at.add(Duration::days(1));

    let ack = Acknowledge {
        matchers,
        created_by: "anag-bridge",
        comment: comment,
        ends_at: ends_at.to_rfc3339(),
        starts_at: starts_at.to_rfc3339(),
    };

    let response = new_http_client()
        .post("http://alertmanager:9093/api/v2/silences")
        .header("Content-Type", "application/json")
        .send_json(&ack)
        .await?;

    if !response.status().is_success() {
        return Err(Error::BadStatus(response.status()));
    }

    Ok(())
}

pub async fn remove_ack(name: String) -> Result<(), Error> {
    let query = [("filter", format!("alertname={}", name))];

    let mut response = new_http_client()
        .get("http://alertmanager:9093/api/v2/silences")
        .header("Content-Type", "application/json")
        .query(&query)?
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(Error::BadStatus(response.status()));
    }

    let silences: Vec<Silence> = response.json().await?;

    for silence in silences {
        if silence.status.state != "active".to_string() {
            continue;
        }

        use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
        const QUERY: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'#').add(b'<').add(b'>');
        let encoded_name = utf8_percent_encode(silence.id.as_str(), QUERY);
        let url = format!("http://alertmanager:9093/api/v2/silence/{}", encoded_name);

        let response = new_http_client().delete(url).send().await?;

        if !response.status().is_success() {
            return Err(Error::BadStatus(response.status()));
        }
    }

    Ok(())
}

impl Alert {
    pub fn select_message(&self) -> String {
        if self.annotations.message.is_some() {
            return self.annotations.message.clone().unwrap();
        }
        if self.annotations.description.is_some() {
            return self.annotations.description.clone().unwrap();
        }
        if self.annotations.summary.is_some() {
            return self.annotations.summary.clone().unwrap();
        }

        "no description".to_string()
    }

    pub fn select_severity<'a>(&'a self) -> String {
        if self.labels.severity.is_some() {
            return match self.labels.severity.clone().unwrap().as_str() {
                "critical" => "CRITICAL",
                "warning" => "WARNING",
                "info" | "none" => "PENDING",
                _ => "UNKNOWN",
            }
            .to_string();
        }

        "UNKNOWN".to_string()
    }

    pub fn select_name<'a>(&'a self) -> String {
        if self.labels.alertname.is_some() {
            return self.labels.alertname.clone().unwrap();
        }

        self.fingerprint.clone()
    }

    pub fn select_acknowledged<'a>(&'a self) -> bool {
        self.status.state == "suppressed"
    }
}

fn new_http_client() -> Client {
    let connector = actix_web::client::Connector::new()
        .timeout(std::time::Duration::from_secs(10))
        .finish();
    actix_web::client::ClientBuilder::new()
        .connector(connector)
        .timeout(std::time::Duration::from_secs(10))
        .finish()
}

#[derive(Deserialize, Debug)]
pub struct Alert {
    pub annotations: Annotations,
    #[serde(rename = "generatorURL")]
    pub generator_url: String,
    pub fingerprint: String,
    pub status: Status,
    pub labels: Labels,
}

#[derive(Deserialize, Debug)]
pub struct Annotations {
    pub description: Option<String>,
    pub summary: Option<String>,
    pub message: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Status {
    pub state: String,
}

#[derive(Deserialize, Debug)]
pub struct Labels {
    pub alertname: Option<String>,
    pub severity: Option<String>,
}

#[derive(Serialize)]
pub struct Acknowledge<'a> {
    pub matchers: Vec<Matcher>,
    #[serde(rename = "createdBy")]
    pub created_by: &'a str,
    pub comment: String,
    #[serde(rename = "endsAt")]
    pub ends_at: String,
    #[serde(rename = "startsAt")]
    pub starts_at: String,
}

#[derive(Serialize)]
pub struct Matcher {
    pub name: String,
    pub value: String,
    #[serde(rename = "isRegex")]
    pub is_regex: bool,
}

#[derive(Serialize)]
pub struct Filter {
    pub filter: [Matcher; 1],
}

#[derive(Deserialize)]
pub struct Silence {
    pub id: String,
    pub status: Status,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("alertmanager responded with status code {0}.")]
    BadStatus(StatusCode),
    #[error("{0}")]
    RequestError(#[from] SendRequestError),
    #[error("{0}")]
    JsonPayloadError(#[from] JsonPayloadError),
    #[error("{0}")]
    QueryEncode(#[from] serde_urlencoded::ser::Error),
}
