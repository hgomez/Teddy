mod conf;
mod handlers;
mod middleware;

use actix_web::guard::{Guard, GuardContext};
use actix_web::middleware::Logger;
use actix_web::web::{self, get, post, Data};
use actix_web::{http, main, App, HttpServer};
use log::info;

use base64::{engine::general_purpose, Engine as _};

#[derive(Clone)]
struct Authorization {
    pub token: String,
}

#[derive(Clone)]
struct AuthorizationGuard;

impl Guard for AuthorizationGuard {
    fn check(&self, req: &GuardContext) -> bool {
        match req.head().headers().get(http::header::AUTHORIZATION) {
            Some(token) => {
                token.to_str().unwrap_or("") == req.app_data::<Authorization>().unwrap().token
            }
            None => false,
        }
    }
}

#[main]
async fn main() -> std::io::Result<()> {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    info!("Starting Teddy");
    let configuration = conf::load_config();
    let address = conf::get_address(&configuration);
    info!("Listening on {}", address);

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(Authorization {
                token: general_purpose::STANDARD.encode(&format!(
                    "{}:{}",
                    configuration.user, configuration.password
                )),
            }))
            .wrap(Logger::default())
            .route("/", web::route().to(handlers::welcome))
            .route("/ping", get().to(handlers::ping))
            .route("/download", get().to(handlers::download))
            .route("/upload", get().to(handlers::upload))
            .route(
                "/exec",
                post().guard(AuthorizationGuard).to(handlers::execute),
            )
    })
    .bind(address)?
    .run()
    .await
}
