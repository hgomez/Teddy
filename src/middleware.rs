// impl<S, B> Transform<S, ServiceRequest> for Authentication
// where
// S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
// S::Future: 'static,
// B: 'static,
// {
// type Response = ServiceResponse<B>;
// type Error = Error;
// type InitError = ();
// type Transform = AuthMiddleware<S>;
// type Future = Ready<Result<Self::Transform, Self::InitError>>;

// fn new_transform(&self, service: S) -> Self::Future {
// ready(Ok(AuthMiddleware {
// service,
// token: self.token.clone(),
// }))
// }
// }

// pub struct AuthMiddleware<S> {
// service: S,
// token: String,
// }

// impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
// where
// S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
// S::Future: 'static,
// B: 'static,
// {
// type Response = ServiceResponse<B>;
// type Error = Error;
// type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

// fn call(&self, req: ServiceRequest) -> Self::Future {
// let token = req
// .headers()
// .get(header::HeaderName::from_static("authorization"))
// .ok_or(error::ErrorUnauthorized(format_err!(
// "Missing authorization header"
// )))
// .and_then(|header_value| parse(header_value).map_err(|e| error::ErrorUnauthorized(e)))
// .unwrap();

// let fut = self.service.call(req);

// let response = {

// if token == self.token {
// Ok(res)
// } else {
// Err(error::ErrorForbidden(format_err!(
// "Invalid username/password"
// )))
// }
// }

// Box::pin(async move {
// let res = fut.await?;
// response
// })
// }

// fn poll_ready(
// &self,
// _ctx: &mut core::task::Context<'_>,
// ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
// todo!()
// }
// }

// fn parse(header: &header::HeaderValue) -> Result<String, failure::Error> {
// let mut parts = header.to_str()?.splitn(2, ' ');
// match parts.next() {
// Some(scheme) if scheme == "Basic" => (),
// _ => return Err(format_err!("Invalid header : No basic authentication")),
// }
// parts
// .next()
// .map(|str| String::from(str))
// .ok_or(format_err!("Invalid basic header : No token"))
// }

// impl Authentication {
    // pub fn new(configuration: &Configuration) -> Self {
        // Authentication {
        // }
    // }
// }
