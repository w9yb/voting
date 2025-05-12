#![feature(map_try_insert, lock_value_accessors)]

mod admin;
mod data;
mod static_page;

use std::{collections::BTreeMap, sync::Mutex};

use actix_web::{HttpResponse, Responder};
use rand::RngCore as _;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let certs_file = std::fs::read("cert.pem").unwrap();
    let key_file = std::fs::read("key.pem").unwrap();

    let tls_certs = rustls_pemfile::certs(&mut certs_file.as_slice())
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let tls_key = rustls_pemfile::pkcs8_private_keys(&mut key_file.as_slice())
        .next()
        .unwrap()
        .unwrap();

    let tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))
        .unwrap();

    let mut rng = rand::rng();
    let key: String = (0..4)
        .flat_map(|_| {
            let n = rng.next_u32();
            [
                n as u8 & 0x1f,
                (n >> 5) as u8 & 0x1f,
                (n >> 10) as u8 & 0x1f,
                (n >> 15) as u8 & 0x1f,
                (n >> 20) as u8 & 0x1f,
                (n >> 25) as u8 & 0x1f,
            ]
        })
        .map(|c| char::from_digit(c.into(), 32).unwrap())
        .collect();

    println!("Key: {}", key);
    let app_data = actix_web::web::Data::new(ElectionData::new(key));

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .app_data(app_data.clone())
            .route(
                "/",
                actix_web::web::get().to(static_page::Static(include_str!("../assets/index.html"))),
            )
            .route(
                "/done",
                actix_web::web::get().to(static_page::Static(include_str!("../assets/done.html"))),
            )
            .route(
                "/error",
                actix_web::web::get().to(static_page::Static(include_str!("../assets/error.html"))),
            )
            .service(ballot_submission)
            .service(admin::check_ballots)
            .service(admin::get_results)
    })
    .bind(("127.0.0.1", 8080))?
    .bind_rustls_0_23(("127.0.0.1", 4443), tls_config)?
    .run()
    .await
}

struct ElectionData {
    key: String,
    ballots: Mutex<BTreeMap<String, Vec<String>>>,
}

impl ElectionData {
    fn new(key: String) -> Self {
        Self {
            key,
            ballots: Mutex::default(),
        }
    }
}

#[actix_web::post("/ballot")]
async fn ballot_submission(
    data: actix_web::web::Data<ElectionData>,
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
