use actix_files::NamedFile;
use actix_web::{web, Result};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct DownloadQueryParams {
    filename: String
}

pub async fn handler(params: web::Query<DownloadQueryParams>) -> Result<NamedFile> {
    Ok(NamedFile::open(&params.filename)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::http::Method;
    use actix_web::test::TestRequest;
    use actix_web::{test, web, App};
    use memchr::memmem::find;

    #[actix_web::test]
    async fn test_download() {
        let req = TestRequest::default().method(Method::POST).uri("/?filename=README.md").to_request();
        let test_app = test::init_service(App::new().route("/", web::post().to(handler))).await;

        let response = test::call_and_read_body(&test_app, req).await;
        assert!(find(&response, b"Teddy").is_some());
    }
}
