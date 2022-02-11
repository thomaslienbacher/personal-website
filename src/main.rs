use std::fs::File;
use std::io::BufReader;
use actix_files::Files;
use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use rustls::internal::pemfile::{certs, pkcs8_private_keys};
use rustls::{NoClientAuth, ServerConfig};

async fn index(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/index.html"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cert_file_path = std::env::var("CERT_FILE").unwrap_or_else(|_| {
        panic!("Please set CERT_FILE environment variable (fullchain.pem)!")
    });
    let key_file_path = std::env::var("KEY_FILE").unwrap_or_else(|_| {
        panic!("Please set KEY_FILE environment variable (privkey.pem)!")
    });

    if std::env::var("RUST_LOG").is_ok() {
        println!("WARNING: Logging is enabled!");
        env_logger::init();
    } else {
        println!("For logging set RUST_LOG to actix_server=debug,actix_web=debug")
    }

    // load ssl keys
    let mut config = ServerConfig::new(NoClientAuth::new());
    let cert_file = &mut BufReader::new(File::open(cert_file_path).unwrap());
    let key_file = &mut BufReader::new(File::open(key_file_path).unwrap());
    let cert_chain = certs(cert_file).unwrap();
    let mut keys = pkcs8_private_keys(key_file).unwrap();
    if keys.is_empty() {
        eprintln!("Could not locate PKCS 8 private keys.");
        std::process::exit(1);
    }
    config.set_single_cert(cert_chain, keys.remove(0)).unwrap();

    println!("Starting https server: 0.0.0.0:443");

    if std::env::var("RUST_LOG").is_ok() {
        HttpServer::new(|| {
            App::new()
                .wrap(middleware::Logger::default())
                .service(web::resource("/").route(web::get().to(index)))
                .service(Files::new("/static", "static"))
        })
            .bind_rustls("0.0.0.0:443", config)?
            .run()
            .await
    } else {
        HttpServer::new(|| {
            App::new()
                .service(web::resource("/").route(web::get().to(index)))
                .service(Files::new("/static", "static"))
        })
            .bind_rustls("0.0.0.0:443", config)?
            .run()
            .await
    }
}