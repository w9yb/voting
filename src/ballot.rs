use actix_web::{HttpResponse, Responder};

use crate::{data, state::ApplicationState};

#[actix_web::get("/")]
pub async fn ballot_form(data: actix_web::web::Data<ApplicationState>) -> impl Responder {
    let mut context = tera::Context::new();
    let candidates = data.list_candidates().await;
    context.insert("candidates", &candidates);
    HttpResponse::Ok().body(data.templates().render("ballot.html", &context).unwrap())
}

#[actix_web::post("/ballot")]
async fn ballot_submission(
    data: actix_web::web::Data<ApplicationState>,
    ballot: actix_web::web::Form<data::Ballot>,
) -> impl Responder {
    match data.add_ballot(ballot.0.callsign, ballot.0.ranking).await {
        Ok(()) => HttpResponse::SeeOther()
            .insert_header(("Location", "/done"))
            .finish(),
        Err(e) => HttpResponse::SeeOther()
            .insert_header(("Location", format!("/error?error={e}")))
            .finish(),
    }
}
