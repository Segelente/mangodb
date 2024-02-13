//! Webpage for blog posts.
use std::fs::read_to_string;
use std::path::Path;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web::web::Data;
use liquid::{object, ObjectView, Template, ValueView};
use mongodb::Client;
use dotenvy;
use tracing::info;
use mongodb::options::ClientOptions;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use crate::example2::queries::{create_post, get_post};

pub(crate) struct AppState {
    client: Mutex<Client>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Post {
    pub title: String,
    pub content: String,
    pub path: String,
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
#[get("/post/{path}")]
async fn post(data: Data<AppState>, path: web::Path<String>) -> impl Responder{
    info!("Getting post with id {}", path.clone());
    let globals = object!(
        {"post": get_post(data.client.lock().await.clone(), path.into_inner() ).await});
    let template = liquid_parse("src/web/post.liquid");
    println!("{:?}", globals);
    let output = template.render(&globals).unwrap();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(output)
}

#[actix_web::main]
pub(crate) async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();
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
            .app_data(app_state)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}