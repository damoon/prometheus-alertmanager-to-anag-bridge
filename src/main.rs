#[macro_use]
extern crate serde;
extern crate env_logger;

use actix_web::{get, web, App, HttpServer, HttpResponse, Responder, web::Query, Result};
use actix_web::middleware::Logger;

#[derive(Deserialize)]
pub struct DetailsRequest {
   style: String,
}

#[derive(Serialize)]
pub struct TacResponse<'a> {
    pub cgi_json_version: &'a str,
    pub icinga_status: IcingaStatus,
    pub tac: Tac<'a>,
}

#[derive(Serialize)]
pub struct Tac<'a> {
    pub tac_overview: TacOverview<'a>,
}

#[derive(Serialize)]
pub struct TacOverview<'a> {
    pub total_services: &'a i32,
    pub services_ok: &'a i32,
    pub services_warning: &'a i32,
}

#[derive(Serialize)]
pub struct ServicesResponse<'a> {
    pub cgi_json_version: &'a str,
    pub icinga_status: IcingaStatus,
    pub status: Status<'a>,
}

#[derive(Serialize)]
pub struct IcingaStatus {
}

#[derive(Serialize)]
pub struct Status<'a> {
    pub service_status: &'a [ServiceStatus<'a>],
    //pub host_status: [HostStatus],
}

#[derive(Serialize)]
pub struct ServiceStatus<'a> {
    pub host_name: &'a str,
    pub host_display_name: &'a str,
    pub service_description: &'a str,
    pub service_display_name: &'a str,
    pub status: &'a str,
    pub last_check: &'a str,
    pub duration: &'a str,
    pub attempts: &'a str,
    pub current_notification_number: &'a i32,
    pub state_type: &'a str,
    pub is_flapping: bool,
    pub in_scheduled_downtime: bool,
    pub active_checks_enabled: bool,
    pub passive_checks_enabled: bool,
    pub notifications_enabled: bool,
    pub has_been_acknowledged: bool,
    pub action_url: &'a str,
    pub notes_url: &'a str,
    pub status_information: &'a str,
}

#[derive(Serialize)]
pub struct HostStatus {
}

// /nagios/cgi-bin/status.cgi?style=servicedetail&embedded&limit=0&serviceprops=262144&servicestatustypes=61&jsonoutput
#[get("/nagios/cgi-bin/status.cgi")]
async fn servicedetail(info: web::Query<DetailsRequest>) -> Result<HttpResponse> {
    let services = &[
        ServiceStatus{
            host_name: "cluster",
            host_display_name: "cluster",
            service_description: "Kubernetes Pods",
            service_display_name: "Kubernetes Pods",
            status: "WARNING",
            last_check: "2020-12-02 17:56:56",
            duration: "4d  7h 38m 52s",
            attempts: "4/4",
            current_notification_number: &1,
            state_type: "HARD",
            is_flapping: false,
            in_scheduled_downtime: false,
            active_checks_enabled: true,
            passive_checks_enabled: true,
            notifications_enabled: true,
            has_been_acknowledged: false,
            action_url: "",
            notes_url: "",
            status_information: "[WARN] 2 pods are in a incomplete state"
        }
    ];
    let hosts = &[HostStatus{}];

    let response = ServicesResponse{
        cgi_json_version: "a",
        icinga_status: IcingaStatus{},
        status: Status{
            service_status: services,
        },
    };

    Ok(HttpResponse::Ok().json(response))
}

#[get("/nagios/cgi-bin/tac.cgi")]
async fn tac() -> Result<HttpResponse> {
    let services = &[
        ServiceStatus{
            host_name: "cluster",
            host_display_name: "cluster",
            service_description: "Kubernetes Pods",
            service_display_name: "Kubernetes Pods",
            status: "WARNING",
            last_check: "2020-12-02 17:56:56",
            duration: "4d  7h 38m 52s",
            attempts: "4/4",
            current_notification_number: &1,
            state_type: "HARD",
            is_flapping: false,
            in_scheduled_downtime: false,
            active_checks_enabled: true,
            passive_checks_enabled: true,
            notifications_enabled: true,
            has_been_acknowledged: false,
            action_url: "",
            notes_url: "",
            status_information: "[WARN] 2 pods are in a incomplete state"
        }
    ];

    let response = TacResponse{
        cgi_json_version: "a",
        icinga_status: IcingaStatus{},
        tac: Tac{
            tac_overview: TacOverview {
                total_services: &6,
                services_ok: &4,
                services_warning: &2,
            }
        },
    };

    Ok(HttpResponse::Ok().json(response))
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(|| { App::new()
            .wrap(Logger::default())
            .service(servicedetail)
            .service(tac)
        })
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
