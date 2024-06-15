use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web::dev::Server;

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("world");
    format!("Hello {}!", &name)
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().finish()
}

pub fn run() -> Result<Server, std::io::Error> {
    // let body = async move {
    //     HttpServer::new(|| {
    //         App::new()
    //             .route("/", web::get().to(greet))
    //             .route("/{name}", web::get().to(greet))
    //             .route("/health_chack", web::get().to(health_check))
    //     })
    //     .bind("127.0.0.1:8000")?
    //     .run()
    //     .await
    // };

    // tokio::runtime::Builder::new_multi_thread()
    //     .enable_all()
    //     .build()
    //     .expect("Failed building the Runtime")
    //     .block_on(body)
    let server = HttpServer::new(|| {
        App::new().route("/health_check", web::get().to(health_check))
    })
        .bind("127.0.0.1:8000")?
        .run();

    Ok(server)
}