extern crate actix_web;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate base64;
extern crate futures;
extern crate simple_logger;

mod conf;
mod handlers;
mod middleware;

use actix_web::http::Method;
use actix_web::middleware::Logger;
use actix_web::{server, App};

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    info!("Starting Teddy");
    let configuration = conf::load_config();
    let address = conf::get_address(&configuration);
    server::new(move || {
        App::new()
            .middleware(Logger::default())
            .middleware(middleware::Authentication::new(&configuration))
            .resource("/", |r| r.method(Method::GET).f(handlers::welcome))
            .resource("/ping", |r| r.method(Method::GET).f(handlers::ping))
            .resource("/download", |r| {
                r.method(Method::GET).with(handlers::download)
            })
            .resource("/upload", |r| r.method(Method::POST).with(handlers::upload))
            .resource("/exec", |r| r.method(Method::POST).with(handlers::execute))
    })
    .bind(address)
    .unwrap()
    .run();
}
