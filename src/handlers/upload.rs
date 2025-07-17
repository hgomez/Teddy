use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_web::HttpResponse;

#[derive(Debug, MultipartForm)]
pub struct UploadForm {
    #[multipart(limit = "100MB")]
    uploaded_file: TempFile,
}

pub async fn handler(MultipartForm(form): MultipartForm<UploadForm>) -> HttpResponse {
    HttpResponse::Ok().body(format!(
        "Uploaded file {}, with size: {}",
        form.uploaded_file.file_name.expect("Should have a name"),
        form.uploaded_file.size
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_multipart::test::create_form_data_payload_and_headers;

    use actix_web::http::Method;
    use actix_web::test::TestRequest;
    use actix_web::{test, web, App};
    use bytes::Bytes;
    use memchr::memmem::find;

    #[actix_web::test]
    async fn test_upload() {
        let (body, headers) = create_form_data_payload_and_headers(
            "uploaded_file", // This MUST be the same field name as in UploadForm !
            Some("lorem.txt".to_owned()),
            Some(mime::TEXT_PLAIN_UTF_8),
            Bytes::from_static(b"Lorem ipsum."),
        );

        let req = TestRequest::default().method(Method::POST);

        // merge header map into existing test request and set multipart body
        let req = headers
            .into_iter()
            .fold(req, |req, hdr| req.insert_header(hdr))
            .set_payload(body)
            .to_request();

        let test_app = test::init_service(App::new().route("/", web::post().to(handler))).await;

        let response = test::call_and_read_body(&test_app, req).await;
        assert!(find(&response, b"Uploaded file lorem.txt, with size: 12").is_some());
    }
}
