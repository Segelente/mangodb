//! Webpage for blog posts.
use std::fs::read_to_string;
use std::path::Path;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web::web::Data;
use liquid::{object, Template};
use mongodb::Client;
use mongodb::options::ClientOptions;
use serde::Serialize;
use tokio::sync::Mutex;
use crate::example2::queries::get_post;

struct AppState {
    client: Mutex<Client>,
}
#[derive(Debug, Serialize)]
struct Post {
    title: String,
    content: String,
    path: String,
}

pub(crate) fn liquid_parse(path: impl AsRef<Path>) -> Template {
    let compiler = liquid::ParserBuilder::with_stdlib()
        .build()
        .expect("Could not build liquid compiler");
    compiler
        .parse(read_to_string(path).unwrap().as_str())
        .unwrap()
}

#[get("/")]
async fn index() -> impl Responder{
    let body = read_to_string("src/web/index.liquid").unwrap();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(body)
}
#[get("/post/{id}")]
async fn post(data: Data<AppState>, id: web::Path<i32>) -> impl Responder{
    let globals = get_post(data.client.lock(), id.into_inner() ).await;
    let template = liquid_parse("src/web/post.liquid");
    let output = template.render(&globals).unwrap();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(output)
}

#[actix_web::main]
pub(crate) async fn main() -> std::io::Result<()> {
    let db_url = &std::env::var("DATABASE_URL").unwrap();
    let  client_options = ClientOptions::parse(db_url).await.unwrap();
    let client = Client::with_options(client_options).unwrap();

    HttpServer::new(move || {
        let app_state = Data::new(AppState {
            client: Mutex::new(client.clone()),
        });
        App::new()
            .service(index)
            .service(post)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}