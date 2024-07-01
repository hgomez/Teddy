use std::path::PathBuf;

use actix_files::NamedFile;
use actix_web::{HttpRequest, Result};

pub async fn handler(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    Ok(NamedFile::open(path)?)
}
