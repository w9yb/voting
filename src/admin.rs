use std::collections::{BTreeMap, BTreeSet};

use actix_web::{HttpResponse, Responder};

use crate::state::{self, ApplicationState};

#[derive(serde::Deserialize)]
struct Key {
    key: String,
}

#[actix_web::get("/admin/check_ballots")]
async fn check_ballots(
    data: actix_web::web::Data<ApplicationState>,
    authentication: actix_web::web::Query<Key>,
) -> impl Responder {
    if data.check_key(&authentication.key) {
        HttpResponse::Ok().json(data.list_ballots().await)
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

#[actix_web::get("/admin/check_candidates")]
async fn check_candidates(
    data: actix_web::web::Data<ApplicationState>,
    authentication: actix_web::web::Query<Key>,
) -> impl Responder {
    if data.check_key(&authentication.key) {
        HttpResponse::Ok().json(data.list_candidates().await)
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

#[derive(serde::Deserialize)]
struct AuthenticatedCandidates {
    key: String,
    candidates: String,
    allow_leave_empty: bool,
}

#[actix_web::get("/admin/set_candidates")]
async fn set_candidates(
    data: actix_web::web::Data<ApplicationState>,
    new_candidates: actix_web::web::Query<AuthenticatedCandidates>,
) -> impl Responder {
    if data.check_key(&new_candidates.key) {
        data.set_candidates(
            new_candidates
                .0
                .candidates
                .split(',')
                .map(|s| s.to_owned())
                .collect(),
            new_candidates.allow_leave_empty,
        )
        .await;
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

#[derive(serde::Serialize)]
struct ElectionResults {
    data: state::Data,
    pairwise_data: Vec<BTreeSet<(String, String)>>,
    winners: BTreeSet<String>,
}

impl ElectionResults {
    pub async fn from_election_data(data: &ApplicationState) -> Self {
        let data = data.take_data().await;

        let candidates: BTreeSet<&str> = data
            .ballots
            .iter()
            .flat_map(|b| b.iter().map(String::as_str))
            .collect();

        // the candidates should be a subset of all available candidates
        assert_eq!(
            candidates.iter().find(|c| !data.candidates.contains(**c)),
            None
        );

        let (candidates_to_num, num_to_candidate): (BTreeMap<&str, u16>, BTreeMap<u16, &str>) =
            candidates
                .into_iter()
                .enumerate()
                .map(|(n, s)| {
                    let n = u16::try_from(n).unwrap();
                    ((s, n), (n, s))
                })
                .unzip();

        let tabulated_data = ranked_pairs::TabulatedData::from_ballots(
            &data
                .ballots
                .iter()
                .map(|ballot| {
                    ballot
                        .iter()
                        .map(|v| *candidates_to_num.get(v.as_str()).unwrap())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
            candidates_to_num.len().try_into().unwrap(),
        )
        .unwrap();

        let pairwise_data = tabulated_data
            .pairwise_results()
            .map(|s| {
                s.iter()
                    .map(|(winner, loser)| {
                        (
                            (*num_to_candidate.get(winner).unwrap()).to_owned(),
                            (*num_to_candidate.get(loser).unwrap()).to_owned(),
                        )
                    })
                    .collect()
            })
            .collect();

        let winners = tabulated_data
            .tally()
            .into_iter()
            .map(|w| num_to_candidate.get(&w).unwrap().to_string())
            .collect();

        Self {
            data,
            pairwise_data,
            winners,
        }
    }
}

#[actix_web::get("/admin/get_results")]
async fn get_results(
    data: actix_web::web::Data<ApplicationState>,
    authentication: actix_web::web::Query<Key>,
) -> impl Responder {
    if data.check_key(&authentication.key) {
        let results = ElectionResults::from_election_data(&data).await;
        HttpResponse::Ok().json(results)
    } else {
        HttpResponse::Unauthorized().finish()
    }
}
