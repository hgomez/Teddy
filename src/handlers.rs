use actix_files::NamedFile;
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::{json::Json as MPJson, MultipartForm};
use actix_web::error::{Error, ErrorInternalServerError};
use actix_web::web::Json;
use actix_web::{HttpRequest, HttpResponse, Responder, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;

pub async fn welcome() -> HttpResponse {
    HttpResponse::Ok().body("Welcome to Teddy, see ya !")
}

pub async fn ping() -> HttpResponse {
    HttpResponse::Ok().body("pong")
}

pub async fn download(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    Ok(NamedFile::open(path)?)
}

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

pub async fn execute(query: Json<CommandQuery>) -> Result<HttpResponse, Error> {
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

#[derive(Debug, Deserialize)]
pub struct Metadata {
    name: String,
}

#[derive(Debug, MultipartForm)]
pub struct UploadForm {
    #[multipart(limit = "100MB")]
    file: TempFile,
    json: MPJson<Metadata>,
}

pub async fn upload(MultipartForm(form): MultipartForm<UploadForm>) -> impl Responder {
    format!(
        "Uploaded file {}, with size: {}",
        form.json.name, form.file.size
    )
}
