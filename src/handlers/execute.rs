use std::{env, ffi::OsString, process::Command};

use actix_web::{error::ErrorInternalServerError, web::Json, Error, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CommandQuery {
    command: String,
    parameters: String,
}

#[derive(Serialize)]
pub struct CommandResponse {
    status: Option<i32>,
    stdout: String,
    stderr: String,
}

pub async fn handler(query: Json<CommandQuery>) -> Result<HttpResponse, Error> {
    Command::new(query.command.replace("\"", ""))
        .env(
            "PATH",
            env::var_os("PATH").unwrap_or_else(|| OsString::from("")),
        )
        .arg(query.parameters.replace("\"", ""))
        .output()
        .map(|output| CommandResponse {
            status: output.status.code(),
            stdout: String::from_utf8(output.stdout)
                .unwrap_or_else(|_| String::from("Can't parse command stdout")),
            stderr: String::from_utf8(output.stderr)
                .unwrap_or_else(|_| String::from("Can't parse command stderr")),
        })
        .map_err(ErrorInternalServerError)
        .map(|command_response| HttpResponse::Ok().json(command_response))
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::http::header::ContentType;
    use actix_web::http::Method;
    use actix_web::test::TestRequest;
    use actix_web::{test, web, App};
    use memchr::memmem::find;

    #[actix_web::test]
    async fn test_download() {
        let req = TestRequest::default()
            .method(Method::POST)
            .insert_header(ContentType(mime::APPLICATION_JSON))
            .set_json(CommandQuery {
                command: "echo".to_owned(),
                parameters: "coucou".to_owned(),
            })
            .to_request();
        let test_app = test::init_service(App::new().route("/", web::post().to(handler))).await;

        let response = test::call_and_read_body(&test_app, req).await;
        assert!(find(&response, b"coucou").is_some());
    }
}
