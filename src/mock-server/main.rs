use actix_web::{App, HttpServer, Responder, post, web};
use env_logger;
use std::env;

mod problem;
mod server;

use server::Server;
use server::{ExploreRequest, GuessRequest, SelectRequest};

async fn hello() -> impl Responder {
    "Hello, world!"
}

#[post("/select")]
async fn select_server(req: web::Json<SelectRequest>, server: web::Data<Server>) -> impl Responder {
    web::Json(server.select(req.into_inner()))
}

#[post("/explore")]
async fn explore_server(
    req: web::Json<ExploreRequest>,
    server: web::Data<Server>,
) -> impl Responder {
    web::Json(server.explore(req.into_inner()))
}

#[post("/guess")]
async fn guess_server(req: web::Json<GuessRequest>, server: web::Data<Server>) -> impl Responder {
    web::Json(server.guess(req.into_inner()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    unsafe {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let server = web::Data::new(Server::new());

    HttpServer::new(move || {
        App::new()
            .app_data(server.clone())
            .route("/", web::get().to(hello))
            .service(select_server)
            .service(explore_server)
            .service(guess_server)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
