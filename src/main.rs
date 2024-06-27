mod conf;
mod handlers;
mod middlewares;

use actix_web::{guard, http, main, middleware, web, App, HttpServer};
use log::info;

use base64::{engine::general_purpose, Engine as _};

#[derive(Clone)]
struct Authorization {
    pub token: String,
}

#[derive(Clone)]
struct AuthorizationGuard;

impl guard::Guard for AuthorizationGuard {
    fn check(&self, ctx: &guard::GuardContext) -> bool {
        match ctx.app_data::<web::Data<Authorization>>() {
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
            .app_data(web::Data::new(Authorization {
                token: general_purpose::STANDARD.encode(&format!(
                    "{}:{}",
                    configuration.user, configuration.password
                )),
            }))
            .wrap(middleware::Logger::default())
            .route("/", web::get().to(handlers::welcome))
            .route("/ping", web::get().to(handlers::ping))
            .service(
                web::scope("/admin")
                    .guard(AuthorizationGuard)
                    .route("/download", web::get().to(handlers::download))
                    .route("/upload", web::get().to(handlers::upload))
                    .route("/exec", web::post().to(handlers::execute)),
            )
    })
    .bind(address)?
    .run()
    .await
}
