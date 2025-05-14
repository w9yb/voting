use actix_web::HttpResponse;

use crate::ApplicationState;

#[derive(Clone)]
pub struct Static(pub &'static str);

impl actix_web::Handler<actix_web::web::Data<ApplicationState>> for Static {
    type Output = HttpResponse;
    type Future = std::future::Ready<HttpResponse>;

    fn call(&self, data: actix_web::web::Data<ApplicationState>) -> Self::Future {
        std::future::ready(
            HttpResponse::Ok().body(
                data.templates
                    .render(self.0, &tera::Context::new())
                    .unwrap(),
            ),
        )
    }
}
