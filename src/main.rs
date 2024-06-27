mod conf;
mod guards;
mod handlers;

use actix_web::{main, middleware, web, App, HttpServer};
use log::info;

#[main]
async fn main() -> std::io::Result<()> {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    info!("Starting Teddy");
    let configuration = conf::load_config();
    let address = conf::get_address(&configuration);
    info!("Listening on {}", address);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(guards::Authorization::new(&configuration)))
            .wrap(middleware::Logger::default())
            .route("/", web::get().to(handlers::welcome))
            .route("/ping", web::get().to(handlers::ping))
            .service(
                web::scope("/admin")
                    .guard(guards::AuthorizationGuard)
                    .route("/download", web::get().to(handlers::download))
                    .route("/upload", web::get().to(handlers::upload))
                    .route("/exec", web::post().to(handlers::execute)),
            )
    })
    .bind(address)?
    .run()
    .await
}
