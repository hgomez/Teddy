use actix_multipart::form::{json::Json, tempfile::TempFile, MultipartForm};
use actix_web::Responder;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Metadata {
    name: String,
}

#[derive(Debug, MultipartForm)]
pub struct UploadForm {
    #[multipart(limit = "100MB")]
    file: TempFile,
    json: Json<Metadata>,
}

pub async fn handler(MultipartForm(form): MultipartForm<UploadForm>) -> impl Responder {
    format!(
        "Uploaded file {}, with size: {}",
        form.json.name, form.file.size
    )
}
