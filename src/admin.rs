use std::collections::{BTreeMap, BTreeSet};

use actix_web::{HttpResponse, Responder};
use rand::seq::SliceRandom;

use crate::ApplicationState;

#[derive(serde::Deserialize)]
struct Key {
    key: String,
}

#[actix_web::get("/admin/check_ballots")]
async fn check_ballots(
    data: actix_web::web::Data<ApplicationState>,
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

#[derive(serde::Deserialize)]
struct AuthenticatedCandidates {
    key: String,
    candidates: String,
}

#[actix_web::get("/admin/set_candidates")]
async fn set_candidates(
    data: actix_web::web::Data<ApplicationState>,
    new_candidates: actix_web::web::Query<AuthenticatedCandidates>,
) -> impl Responder {
    if new_candidates.key == data.key {
        let mut candidates = data.candidates.write().unwrap();
        let mut ballots = data.ballots.lock().unwrap();
        *candidates = new_candidates
            .0
            .candidates
            .split(',')
            .map(|s| s.to_owned())
            .collect();
        ballots.clear();
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

#[derive(serde::Serialize)]
struct ElectionResults {
    people: Vec<String>,
    votes: Vec<Vec<String>>,
    winners: BTreeSet<String>,
}

impl ElectionResults {
    pub fn from_election_data(data: &ApplicationState) -> Self {
        let ballots = data.ballots.replace(BTreeMap::default()).unwrap();
        let (people, mut votes): (Vec<_>, Vec<_>) = ballots.into_iter().unzip();
        votes.shuffle(&mut rand::rng());

        let candidates: BTreeSet<&String> = votes.iter().flatten().collect();
        let (candidates_to_num, num_to_candidate): (
            BTreeMap<&String, u16>,
            BTreeMap<u16, &String>,
        ) = candidates
            .into_iter()
            .enumerate()
            .map(|(n, s)| {
                let n = u16::try_from(n).unwrap();
                ((s, n), (n, s))
            })
            .unzip();
        let winners = ranked_pairs::tally(
            &votes
                .iter()
                .map(|ballot| {
                    ballot
                        .iter()
                        .map(|v| *candidates_to_num.get(v).unwrap())
                        .collect()
                })
                .collect::<Vec<Vec<u16>>>(),
            candidates_to_num.len().try_into().unwrap(),
        )
        .unwrap()
        .into_iter()
        .map(|w| num_to_candidate.get(&w).unwrap().to_string())
        .collect();

        Self {
            people,
            votes,
            winners,
        }
    }
}

#[actix_web::get("/admin/get_results")]
async fn get_results(
    data: actix_web::web::Data<ApplicationState>,
    authentication: actix_web::web::Query<Key>,
) -> impl Responder {
    if authentication.key == data.key {
        let results = ElectionResults::from_election_data(&data);
        HttpResponse::Ok().json(results)
    } else {
        HttpResponse::Unauthorized().finish()
    }
}
