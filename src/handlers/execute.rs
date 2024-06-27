use std::{env, ffi::OsString, process::Command};

use actix_web::{error::ErrorInternalServerError, web::Json, Error, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
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
        .map_err(|e| ErrorInternalServerError(e))
        .map(|command_response| HttpResponse::Ok().json(command_response))
}
