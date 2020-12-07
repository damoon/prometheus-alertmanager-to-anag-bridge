#[macro_use]
extern crate serde;
extern crate env_logger;

use actix_web::middleware::Logger;
use actix_web::{get, web, App, HttpResponse, HttpServer, Result};

#[derive(Deserialize)]
pub struct DetailsRequest {
    style: String,
}

#[derive(Serialize)]
pub struct TacResponse<'a> {
    pub cgi_json_version: &'a str,
    pub icinga_status: IcingaStatus,
    pub tac: Tac,
}

#[derive(Serialize)]
pub struct Tac {
    pub tac_overview: TacOverview,
}

#[derive(Serialize)]
pub struct TacOverview {}

#[derive(Serialize)]
pub struct ServicesResponse<'a> {
    pub cgi_json_version: &'a str,
    pub icinga_status: IcingaStatus,
    pub status: Status<'a>,
}

#[derive(Serialize)]
pub struct IcingaStatus {}

#[derive(Serialize)]
pub struct Status<'a> {
    pub service_status: Vec<ServiceStatus<'a>>,
    //pub host_status: [HostStatus],
}

#[derive(Serialize)]
pub struct ServiceStatus<'a> {
    pub host_name: &'a str,
    pub host_display_name: &'a str,
    pub service_description: &'a str,
    pub service_display_name: String,
    pub status: String,
    pub last_check: &'a str,
    pub duration: &'a str,
    pub attempts: &'a str,
    pub current_notification_number: i32,
    pub state_type: &'a str,
    pub is_flapping: bool,
    pub in_scheduled_downtime: bool,
    pub active_checks_enabled: bool,
    pub passive_checks_enabled: bool,
    pub notifications_enabled: bool,
    pub has_been_acknowledged: bool,
    pub action_url: &'a str,
    pub notes_url: &'a str,
    pub status_information: String,
}

#[derive(Serialize)]
pub struct HostStatus {}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(servicedetail)
            .service(tac)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

impl alertmanager::Alert {
    fn select_message(&self) -> String {
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

    fn select_severity<'a>(&'a self) -> String {
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

    fn select_name<'a>(&'a self) -> String {
        if self.labels.alertname.is_some() {
            return self.labels.alertname.clone().unwrap();
        }

        self.fingerprint.clone()
    }
}

fn new_service_status<'a>(name: String, severity: String, details: String) -> ServiceStatus<'a> {
    ServiceStatus {
        host_name: "unknown",
        host_display_name: "",
        service_description: "",
        service_display_name: name,
        status: severity,
        last_check: "",
        duration: "",
        attempts: "",
        current_notification_number: 0,
        state_type: "HARD",
        is_flapping: false,
        in_scheduled_downtime: false,
        active_checks_enabled: true,
        passive_checks_enabled: true,
        notifications_enabled: true,
        has_been_acknowledged: false,
        action_url: "",
        notes_url: "",
        status_information: details,
    }
}

fn indicate_watchdog_missing(alerts: &Vec<alertmanager::Alert>, services: &mut Vec<ServiceStatus>) {
    let missing = alerts.into_iter().all(|a| a.select_name() != "Watchdog");
    if missing {
        services.push(new_service_status(
            "Watchdog missing".to_string(),
            "CRITICAL".to_string(),
            "Watchdog alert is missing".to_string(),
        ));
    }
}

// /nagios/cgi-bin/status.cgi?style=servicedetail&embedded&limit=0&serviceprops=262144&servicestatustypes=61&jsonoutput
#[get("/nagios/cgi-bin/status.cgi")]
async fn servicedetail(info: web::Query<DetailsRequest>) -> Result<HttpResponse> {
    let mut services = Vec::<ServiceStatus>::new();

    if info.style == "servicedetail" {
        let alerts = alertmanager::alerts().await?;

        services = alerts
            .iter()
            .filter(|a| {
                let name = a.select_name();
                name != "Watchdog"
            })
            .map(|a| {
                let name = a.select_name();
                let severity = a.select_severity();
                let msg = a.select_message();

                new_service_status(name, severity, msg)
            })
            .collect();

        indicate_watchdog_missing(&alerts, &mut services);
    }

    let response = ServicesResponse {
        cgi_json_version: "a",
        icinga_status: IcingaStatus {},
        status: Status {
            service_status: services,
        },
    };

    Ok(HttpResponse::Ok().json(response))
}

impl actix_web::ResponseError for alertmanager::Error {}

#[get("/nagios/cgi-bin/tac.cgi")]
async fn tac() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body("{}"))
}

mod alertmanager {
    use actix_web::client::{Client, JsonPayloadError, SendRequestError};
    use http::StatusCode;
    use std::time::Duration;
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum Error {
        #[error("alertmanager responded with status code {0}.")]
        BadStatus(StatusCode),
        #[error("{0}")]
        RequestError(#[from] SendRequestError),
        #[error("{0}")]
        JsonPayloadError(#[from] JsonPayloadError),
    }

    #[derive(Deserialize, Debug)]
    pub struct Alert {
        pub annotations: Annotations,
        #[serde(rename = "generatorURL")]
        pub generator_url: String,
        pub fingerprint: String,
        pub labels: Labels,
    }

    #[derive(Deserialize, Debug)]
    pub struct Annotations {
        pub description: Option<String>,
        pub summary: Option<String>,
        pub message: Option<String>,
    }

    #[derive(Deserialize, Debug)]
    pub struct Labels {
        pub alertname: Option<String>,
        pub severity: Option<String>,
    }

    pub async fn alerts() -> Result<Vec<Alert>, Error> {
        let mut response = Client::default()
            .get("http://alertmanager:9093/api/v2/alerts")
            .header("Content-Type", "application/json")
            .timeout(Duration::from_secs(5))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::BadStatus(response.status()));
        }

        let alerts: Vec<Alert> = response.json().await?;

        print!("alerts: {:?}", alerts);

        Ok(alerts)
    }
}
