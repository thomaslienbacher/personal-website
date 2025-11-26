use actix_files::Files;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, middleware, Responder, web};

async fn index(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/index.html"))
}

async fn robots(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../res/robots.txt"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::var("RUST_LOG").is_ok() {
        println!("WARNING: Logging is enabled!");
        env_logger::init();
    } else {
        println!("For logging set RUST_LOG to actix_server=debug,actix_web=debug")
    }

    println!("Starting server at http://localhost:8001");

    if std::env::var("RUST_LOG").is_ok() {
        HttpServer::new(|| {
            App::new()
                .wrap(middleware::Logger::default())
                .wrap(middleware::Compress::default())
                .service(web::resource("/robots.txt").route(web::get().to(robots)))
                .service(web::resource("/").route(web::get().to(index)))
                .service(Files::new("/static", "static").prefer_utf8(true))
        })
            .bind("localhost:8001")?
            .run()
            .await
    } else {
        HttpServer::new(|| {
            App::new()
                .wrap(middleware::Compress::default())
                .service(web::resource("/robots.txt").route(web::get().to(robots)))
                .service(web::resource("/").route(web::get().to(index)))
                .service(Files::new("/static", "static").prefer_utf8(true))
        })
            .bind("localhost:8001")?
            .run()
            .await
    }
}
