use actix_web::{guard, http, web};
use log::info;

#[derive(Clone)]
pub struct Authorization {
    pub token: String,
}

#[derive(Clone)]
pub struct AuthorizationGuard;

impl guard::Guard for AuthorizationGuard {
    fn check(&self, ctx: &guard::GuardContext) -> bool {
        match ctx.app_data::<web::Data<Authorization>>() {
            Some(app_data_authorization) => {
                match ctx.head().headers().get(http::header::AUTHORIZATION) {
                    Some(token) => {
                        let request_token = token.to_str().unwrap();
                        let result =
                            token.to_str().unwrap_or("undefined") == app_data_authorization.token;
                        if !result {
                            info!("Authorization token {} is invalid", request_token);
                        }
                        result
                    }
                    None => false,
                }
            }
            None => panic!("No authorization token defined server-side, panicking now!"),
        }
    }
}

#[cfg(test)]
mod tests {
    use actix_web::{http, test, web, App, HttpResponse};

    #[actix_web::test]
    async fn authorization_guard() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(crate::guards::Authorization {
                    token: "foo".to_owned(),
                }))
                .route(
                    "/",
                    web::route()
                        .guard(crate::guards::AuthorizationGuard)
                        .to(HttpResponse::Ok),
                ),
        )
        .await;

        let req = test::TestRequest::default()
            .insert_header((http::header::AUTHORIZATION, "foo"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let req = test::TestRequest::default()
            .insert_header((http::header::AUTHORIZATION, "bar"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
    }
}
