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

