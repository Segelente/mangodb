//! Webpage for blog posts.
use std::fs::read_to_string;
use std::path::Path;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web::cookie::time::macros::date;
use actix_web::web::{Data, patch};
use liquid::{object, ObjectView, Template, ValueView};
use mongodb::Client;
use dotenvy;
use tracing::info;
use mongodb::options::ClientOptions;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use crate::example2::queries::{create_comment, create_post, get_post};

pub(crate) struct AppState {
    client: Mutex<Client>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Post {
    pub title: String,
    pub content: String,
    pub path: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Comment {
    pub author: String,
    pub text: String,
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
    let client = data.client.lock().await.clone();
    info!("Getting post with id {}", path.clone());
    let globals = object!(
        {"post": get_post(client, path.into_inner() ).await});
    let template = liquid_parse("src/web/post.liquid");
    println!("{:?}", globals);
    let output = template.render(&globals).unwrap();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(output)
}

#[post("/post/create_post")]
async fn create_post_page(data: Data<AppState>, mut post_json: web::Json<Post>) -> impl Responder {
    let client = data.client.lock().await.clone();
    let new_post = post_json.into_inner();
    let new_post: Post = Post {
        title: new_post.clone().title,
        content: new_post.clone().content,
        path: new_post.clone().path
    };
    create_post(client, new_post).await;
    HttpResponse::Ok()
}
pub struct RequestComment {
    pub author: String,
    pub text: String,
}
#[post("/post/{path}")]
async fn create_comment_page(data: Data<AppState>, comment_json: web::Json<RequestComment>, path: web::Path<String>) -> impl Responder{
    let client = data.client.lock().await.clone();
    let request_comment = comment_json.into_inner();
    let comment = Comment {
        author: request_comment.author,
        text: request_comment.text,
        path: path.to_string()
    };
    create_comment(client, comment).await;
    HttpResponse::Ok()
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
            .service(create_post_page)
            .app_data(app_state)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}