use actix_web::{App, Error, HttpServer, Responder, error::ErrorBadRequest, post, web};
use std::env;

use icfpc2025::problem;
mod server;

use server::Server;

use server::{ExploreRequest, GuessRequest, SelectRequest};

async fn hello() -> impl Responder {
    "Hello, world!"
}

#[post("/select")]
async fn select_server(
    req: web::Json<SelectRequest>,
    server: web::Data<Server>,
) -> Result<impl Responder, Error> {
    match server.select(req.into_inner()) {
        Ok(response) => Ok(web::Json(response)),
        Err(e) => Err(ErrorBadRequest(e)),
    }
}

#[post("/explore")]
async fn explore_server(
    req: web::Json<ExploreRequest>,
    server: web::Data<Server>,
) -> Result<impl Responder, Error> {
    match server.explore(req.into_inner()) {
        Ok(response) => Ok(web::Json(response)),
        Err(e) => Err(ErrorBadRequest(e)),
    }
}

#[post("/guess")]
async fn guess_server(
    req: web::Json<GuessRequest>,
    server: web::Data<Server>,
) -> Result<impl Responder, Error> {
    match server.guess(req.into_inner(), false) {
        Ok(response) => Ok(web::Json(response)),
        Err(e) => Err(ErrorBadRequest(e)),
    }
}

#[post("/guess-keep")]
async fn guess_keep_server(
    req: web::Json<GuessRequest>,
    server: web::Data<Server>,
) -> Result<impl Responder, Error> {
    match server.guess(req.into_inner(), true) {
        Ok(response) => Ok(web::Json(response)),
        Err(e) => Err(ErrorBadRequest(e)),
    }
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
            .service(guess_keep_server)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
