mod conf;
mod guards;
mod handlers;

use actix_web::{main, middleware, web, App, HttpServer};
use actix_web_metrics::ActixWebMetricsBuilder;
use log::info;
use metrics_exporter_prometheus::PrometheusBuilder;

#[main]
async fn main() -> std::io::Result<()> {
    let metrics = ActixWebMetricsBuilder::new().build().unwrap();
    // Install Prometheus exporter to read ActixWebMetricsBuilder data
    PrometheusBuilder::new().install().unwrap();

    simple_logger::init_with_level(log::Level::Info).unwrap();

    info!("Starting Teddy");
    let configuration = conf::load_config();
    let address = conf::get_address(&configuration);
    info!("Listening on {}", address);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(guards::Authorization::new(&configuration)))
            // Install metrics recorder on Actix Tokio thread
            .wrap(metrics.clone())
            .wrap(middleware::Logger::default())
            .route("/", web::get().to(handlers::welcome::handler))
            .route("/ping", web::get().to(handlers::ping::handler))
            .service(
                web::scope("/admin")
                    .guard(guards::AuthorizationGuard)
                    .route("/download", web::post().to(handlers::download::handler))
                    .route("/upload", web::post().to(handlers::upload::handler))
                    .route("/exec", web::post().to(handlers::execute::handler)),
            )
    })
    .bind(address)?
    .run()
    .await
}
