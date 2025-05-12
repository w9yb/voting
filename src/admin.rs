use std::collections::BTreeMap;

use actix_web::{HttpResponse, Responder};
use rand::seq::SliceRandom;

use crate::ElectionData;

#[derive(serde::Deserialize)]
struct Key {
    key: String,
}

#[actix_web::get("/admin/check_ballots")]
async fn check_ballots(
    data: actix_web::web::Data<ElectionData>,
    authentication: actix_web::web::Query<Key>,
) -> impl Responder {
    if authentication.key == data.key {
        HttpResponse::Ok().body(format!(
            "{:?}\n",
            data.ballots.lock().unwrap().keys().collect::<Vec<_>>()
        ))
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

#[derive(serde::Serialize)]
struct ElectionResults {
    people: Vec<String>,
    votes: Vec<Vec<String>>,
}

impl ElectionResults {
    pub fn from_election_data(data: &ElectionData) -> Self {
        let ballots = data.ballots.replace(BTreeMap::default()).unwrap();
        let (people, mut votes): (Vec<_>, Vec<_>) = ballots.into_iter().unzip();
        votes.shuffle(&mut rand::rng());
        Self { people, votes }
    }
}

#[actix_web::get("/admin/get_results")]
async fn get_results(
    data: actix_web::web::Data<ElectionData>,
    authentication: actix_web::web::Query<Key>,
) -> impl Responder {
    if authentication.key == data.key {
        let results = ElectionResults::from_election_data(&data);
        HttpResponse::Ok().json(results)
    } else {
        HttpResponse::Unauthorized().finish()
    }
}
