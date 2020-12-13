#[macro_use]
extern crate serde;
extern crate env_logger;

use actix_web::middleware::Logger;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Result};
use clap::Arg;
use config;

mod alertmanager;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    HttpServer::new(|| {
        let matches = clap::App::new("Patab - Prometheus alertmanager to Anag bridge")
            .version("0.0.1")
            .author("David Sauer <davedamoon@gmail.com>")
            .about("Creates a Nagios endpoints and proxies requests to Prometheus alertmanager.")
            .arg(
                Arg::with_name("config")
                    .short("c")
                    .long("config")
                    .value_name("FILE")
                    .help("Set config file.")
                    .default_value("patam.toml")
                    .env("PATAM_CONFIG")
                    .takes_value(true),
            )
            .get_matches();
        let config = matches.value_of("config").unwrap();
        let mut settings = config::Config::default();
        settings
            .merge(config::File::with_name(config))
            .expect("unable to read config");

        let endpoint = settings.get_str("endpoint").expect("endpoint is missing");
        let username = match settings.get_str("username") {
            Err(config::ConfigError::NotFound(_)) => None,
            Err(v) => panic!(v),
            Ok(v) => Some(v),
        };
        let password = match settings.get_str("password") {
            Err(config::ConfigError::NotFound(_)) => None,
            Err(v) => panic!(v),
            Ok(v) => Some(v),
        };

        App::new()
            .data(AppState {
                client: alertmanager::new(endpoint, username, password),
            })
            .wrap(Logger::default())
            .service(servicedetail)
            .service(cmd)
            .service(tac)
            .service(healthz)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

struct AppState {
    client: alertmanager::Alertmanager,
}

impl actix_web::ResponseError for alertmanager::Error {}

#[get("/nagios/cgi-bin/status.cgi")]
async fn servicedetail(
    data: web::Data<AppState>,
    info: web::Query<DetailsRequest>,
) -> Result<HttpResponse> {
    let mut services = Vec::<ServiceStatus>::new();

    if info.style == "servicedetail" {
        let alerts = &data.client.alerts().await?;

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
                let ack = a.select_acknowledged();

                new_service_status(name, severity, msg, ack)
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

fn indicate_watchdog_missing(alerts: &Vec<alertmanager::Alert>, services: &mut Vec<ServiceStatus>) {
    let missing = alerts.into_iter().all(|a| a.select_name() != "Watchdog");
    if missing {
        services.push(new_service_status(
            "Watchdog missing".to_string(),
            "CRITICAL".to_string(),
            "Watchdog alert is missing".to_string(),
            false,
        ));
    }
}

fn new_service_status<'a>(
    name: String,
    severity: String,
    details: String,
    acknowledged: bool,
) -> ServiceStatus<'a> {
    ServiceStatus {
        host_name: "unknown",
        host_display_name: "",
        service_description: name.clone(),
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
        has_been_acknowledged: acknowledged,
        action_url: "",
        notes_url: "",
        status_information: details,
    }
}

#[post("/nagios/cgi-bin/cmd.cgi")]
async fn cmd(data: web::Data<AppState>, form: web::Form<AckCmd>) -> Result<HttpResponse> {
    match form.cmd_typ {
        34 => {
            let service = form.service.clone();
            let comment = form.com_data.clone().expect("comment missing");

            &data.client.ack(service, comment).await?;

            Ok(HttpResponse::Ok().body(
                "Your command requests were successfully submitted to Icinga for processing.",
            ))
        }

        52 => {
            let service = form.service.clone();
            &data.client.remove_ack(service).await?;

            Ok(HttpResponse::Ok().body(
                "Your command requests were successfully submitted to Icinga for processing.",
            ))
        }

        _ => Ok(HttpResponse::NotAcceptable().body("Command type not implemented.")),
    }
}

#[get("/nagios/cgi-bin/tac.cgi")]
async fn tac() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body("{}"))
}

#[get("/healthz")]
async fn healthz() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().body(""))
}

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
}

#[derive(Serialize)]
pub struct ServiceStatus<'a> {
    pub host_name: &'a str,
    pub host_display_name: &'a str,
    pub service_description: String,
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

#[derive(Deserialize, Debug)]
struct AckCmd {
    cmd_typ: i32,
    service: String,
    com_data: Option<String>,
}
