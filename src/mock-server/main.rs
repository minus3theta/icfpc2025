use actix_web::{web, App, HttpServer, Responder, post};

mod server;
use server::{SelectRequest, ExploreRequest, GuessRequest};
use server::{select, explore, guess};

async fn hello() -> impl Responder {
  "Hello, world!"
}

#[post("/select")]
async fn select_server(req: web::Json<SelectRequest>) -> impl Responder {
  web::Json(select(req.into_inner()).await)
}

#[post("/explore")]
async fn explore_server(req: web::Json<ExploreRequest>) -> impl Responder {
  web::Json(explore(req.into_inner()).await)
}

#[post("/guess")]
async fn guess_server(req: web::Json<GuessRequest>) -> impl Responder {
  web::Json(guess(req.into_inner()).await)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  HttpServer::new(|| App::new()
      .route("/", web::get().to(hello))
      .service(select_server)
      .service(explore_server)
      .service(guess_server)
    )
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
