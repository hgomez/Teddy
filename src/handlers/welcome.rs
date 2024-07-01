use actix_web::HttpResponse;

pub async fn handler() -> HttpResponse {
    HttpResponse::Ok().body("Welcome to Teddy, see ya !")
}

#[cfg(test)]
mod tests {
    use actix_web::{http::header::ContentType, test, web, App};

    use crate::handlers::welcome::handler;

    #[actix_web::test]
    async fn test_index_get() {
        let app = test::init_service(App::new().route("/", web::route().to(handler))).await;
        let req = test::TestRequest::default()
            .insert_header(ContentType::plaintext())
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
