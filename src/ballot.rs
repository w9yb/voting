use std::ops::Deref as _;

use actix_web::{HttpResponse, Responder};

use crate::{ApplicationState, data};

#[actix_web::get("/")]
pub async fn ballot_form(data: actix_web::web::Data<ApplicationState>) -> impl Responder {
    let mut context = tera::Context::new();
    let candidates = data.candidates.read().unwrap();
    context.insert("candidates", candidates.deref());
    HttpResponse::Ok().body(data.templates.render("ballot.html", &context).unwrap())
}

#[actix_web::post("/ballot")]
async fn ballot_submission(
    data: actix_web::web::Data<ApplicationState>,
    ballot: actix_web::web::Form<data::Ballot>,
) -> impl Responder {
    match data
        .ballots
        .lock()
        .unwrap()
        .try_insert(ballot.0.callsign, ballot.0.ranking)
    {
        Ok(_) => HttpResponse::SeeOther()
            .insert_header(("Location", "/done"))
            .finish(),
        Err(_) => HttpResponse::SeeOther()
            .insert_header(("Location", "/error"))
            .finish(),
    }
}
