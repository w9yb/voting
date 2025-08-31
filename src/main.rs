mod admin;
mod ballot;
mod data;
mod state;
mod static_page;

use copypasta::ClipboardProvider as _;
use rand::RngCore as _;
use sha2::Digest as _;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let mut certificate_params = rcgen::CertificateParams::default();
    certificate_params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "w9yb-voting");

    let signing_key = rcgen::KeyPair::generate_for(&rcgen::PKCS_ECDSA_P521_SHA512)
        .expect("creating a signing key to succeed");
    let certificate = certificate_params.self_signed(&signing_key).unwrap();

    print!("Certificate fingerprint (SHA-256): ");
    for b in sha2::Sha256::digest(certificate.der()) {
        print!("{b:02X}");
    }
    println!();

    let tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            vec![certificate.der().to_owned()],
            rustls::pki_types::PrivateKeyDer::Pkcs8(signing_key.serialize_der().into()),
        )
        .unwrap();

    let mut rng = rand::rng();
    let key: String = (0..9)
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

    let mut clipboard_ctx =
        copypasta::ClipboardContext::new().expect("getting clipboard context to succeed");
    clipboard_ctx
        .set_contents(key.clone())
        .expect("setting clipboard contents to succeed");
    println!("Key copied to clipboard");

    let app_data = actix_web::web::Data::new(state::ApplicationState::new(key));

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .app_data(app_data.clone())
            .route(
                "/done",
                actix_web::web::get().to(static_page::Static("done.html")),
            )
            .service(error_page)
            .service(ballot::ballot_form)
            .service(ballot::ballot_submission)
            .service(admin::check_ballots)
            .service(admin::check_candidates)
            .service(admin::get_results)
            .service(admin::set_candidates)
    })
    .bind_rustls_0_23(("0.0.0.0", 4443), tls_config)?
    .run()
    .await
}

#[derive(serde::Deserialize)]
struct ErrorPageParams {
    error: Option<String>,
}

#[actix_web::get("/error")]
pub async fn error_page(
    data: actix_web::web::Data<state::ApplicationState>,
    params: actix_web::web::Query<ErrorPageParams>,
) -> impl actix_web::Responder {
    let mut context = tera::Context::new();
    context.insert(
        "error",
        if let Some(ref e) = params.error {
            e.as_str()
        } else {
            "No additional information."
        },
    );
    actix_web::HttpResponse::Ok().body(data.templates().render("error.html", &context).unwrap())
}
