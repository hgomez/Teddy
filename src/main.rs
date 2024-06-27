mod conf;
mod handlers;
mod middleware;

use actix_web::guard::{Guard, GuardContext};
use actix_web::middleware::Logger;
use actix_web::web::{get, post, Data};
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
    fn check(&self, ctx: &GuardContext) -> bool {
        match ctx.app_data::<Data<Authorization>>() {
            Some(app_data_authorization) => {
                match ctx.head().headers().get(http::header::AUTHORIZATION) {
                    Some(token) => {
                        let request_token = token.to_str().unwrap();
                        let result =
                            token.to_str().unwrap_or("undefined") == app_data_authorization.token;
                        if !result {
                            info!("Authorization token {} is invalid", request_token);
                        }
                        result
                    }
                    None => false,
                }
            }
            None => panic!("No authorization token defined server-side, panicking now!"),
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
            .route("/", get().to(handlers::welcome))
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
