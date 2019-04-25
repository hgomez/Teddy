extern crate actix_web;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate futures;

mod conf;
mod handlers;

use actix_web::http::Method;
use actix_web::{server, App};

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    info!("Starting Teddy");
    let configuration = conf::load_config();
    server::new(||
        App::new()
            .resource("/", |r| r.method(Method::GET).f(handlers::welcome))
            .resource("/ping", |r| r.method(Method::GET).f(handlers::ping))
            .resource("/download", |r| r.method(Method::GET).with(handlers::download))
            .resource("/upload", |r| r.method(Method::POST).with(handlers::upload))
    ).bind(conf::get_address(&configuration))
        .unwrap()
        .run();
}
