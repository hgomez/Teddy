use http::header;
use actix_web::{http, error, HttpRequest, HttpResponse, Result};
use actix_web::middleware::{Middleware, Started, Response};
use std::time::Instant;
use crate::conf::Configuration;
use base64;
use std::str;

pub struct ResponseTime {}

impl<S> Middleware<S> for ResponseTime {
    fn start(&self, req: &HttpRequest<S>) -> Result<Started> {
        req.request().extensions_mut().insert(Instant::now());
        Ok(Started::Done)
    }

    fn response(&self, req: &HttpRequest<S>, resp: HttpResponse)
                -> Result<Response> {
        req.request().extensions().get::<Instant>()
            .map(|instant| info!("Elapsed time on {} for {}, : {} ms", req.path(), req.connection_info().remote().unwrap_or_else(|| "unknown"), instant.elapsed().as_millis()));
        Ok(Response::Done(resp))
    }
}

impl Default for ResponseTime {
    fn default() -> Self {
        ResponseTime {}
    }
}

pub struct Authentication {
    username: String,
    password: String
}

impl<S> Middleware<S> for Authentication {
    fn start(&self, req: &HttpRequest<S>) -> Result<Started> {
        // don't validate CORS pre-flight requests
        if req.method() == "OPTIONS" {
            return Ok(Started::Done);
        }

        let (username, password) = req.headers().get(header::HeaderName::from_static("authorization"))
            .ok_or(error::ErrorUnauthorized(format_err!("Missing authorization header")))
            .and_then(|header_value| parse(header_value).map_err(|e| error::ErrorUnauthorized(e)))?;

        if username == self.username && password == self.password {
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

fn parse(header: &header::HeaderValue) -> Result<(String, String), failure::Error> {
    let mut parts = header.to_str()?.splitn(2, ' ');
    match parts.next() {
        Some(scheme) if scheme == "Basic" => (),
        _ => return Err(format_err!("Invalid header : No basic authentication")),
    }
    let encoded = parts.next().ok_or(format_err!("Invalid basic header : No encoded part"))?;
    let decoded = base64::decode(encoded)?;
    let mut credentials = str::from_utf8(&decoded)?
        .splitn(2, ':');
    let username = credentials.next()
        .ok_or(format_err!("Invalid basic header : No username"))
        .map(|username| username.to_string())?;
    let password = credentials.next()
        .ok_or(format_err!("Invalid basic header : No password"))
        .map(|password| password.to_string())?;
    Ok((username, password))
}

impl Authentication {
    pub fn new(configuration: &Configuration) -> Self {
        Authentication{
            username: configuration.user.clone(),
            password: configuration.password.clone()
        }
    }
}