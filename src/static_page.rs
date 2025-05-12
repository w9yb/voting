use actix_web::HttpResponse;

#[derive(Clone)]
pub struct Static(pub &'static str);

impl actix_web::Handler<()> for Static {
    type Output = HttpResponse;
    type Future = std::future::Ready<HttpResponse>;

    fn call(&self, _: ()) -> Self::Future {
        std::future::ready(HttpResponse::Ok().body(self.0))
    }
}
