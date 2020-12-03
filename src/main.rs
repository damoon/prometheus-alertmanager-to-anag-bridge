extern crate env_logger;

use actix_web::{get, web, App, HttpServer, Responder};
use actix_web::middleware::Logger;

#[get("/{id}/{name}/index.html")]
async fn index(web::Path((id, name)): web::Path<(u32, String)>) -> impl Responder {
    format!("Hello, {}! id:{}", name, id)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(|| { App::new()
            .wrap(Logger::default())
            .service(index)
        })
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
