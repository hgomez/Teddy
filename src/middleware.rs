use http::header;
use actix_web::{http, error, HttpRequest, HttpResponse, Result};
use actix_web::middleware::{Middleware, Started, Response};
use crate::conf::Configuration;
use base64::{Engine as _, engine::general_purpose};

pub struct Authentication {
    token: String
}

impl<S> Middleware<S> for Authentication {
    fn start(&self, req: &HttpRequest<S>) -> Result<Started> {
        let token = req.headers().get(header::HeaderName::from_static("authorization"))
            .ok_or(error::ErrorUnauthorized(format_err!("Missing authorization header")))
            .and_then(|header_value| parse(header_value).map_err(|e| error::ErrorUnauthorized(e)))?;

        if token == self.token {
            Ok(Started::Done)
        } else {
            Err(error::ErrorForbidden(format_err!("Invalid username/password")))
        }
    }

    fn response(&self, _: &HttpRequest<S>, resp: HttpResponse)
                -> Result<Response> {
        Ok(Response::Done(resp))
    }
}

fn parse(header: &header::HeaderValue) -> Result<String, failure::Error> {
    let mut parts = header.to_str()?.splitn(2, ' ');
    match parts.next() {
        Some(scheme) if scheme == "Basic" => (),
        _ => return Err(format_err!("Invalid header : No basic authentication")),
    }
    parts.next().map(|str| String::from(str)).ok_or(format_err!("Invalid basic header : No token"))
}

impl Authentication {
    pub fn new(configuration: &Configuration) -> Self {
        Authentication{
            token: general_purpose::STANDARD.encode(&format!("{}:{}", configuration.user, configuration.password))
        }
    }
}
