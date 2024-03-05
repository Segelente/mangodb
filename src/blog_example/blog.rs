//! Webpage for blog posts.
use std::fs::read_to_string;
use std::path::Path;

use actix_web::cookie::time::macros::date;
use actix_web::web::Data;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use dotenvy;
use liquid::{object, Template};
use mongodb::bson::{doc, Bson};
use mongodb::options::ClientOptions;
use mongodb::{Client, Collection, Database};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::blog_example::queries::{
    create_comment, create_post, delete_post_query, get_all_posts, get_post,
};

pub(crate) struct AppState {
    database: Mutex<Database>,
}
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Post {
    pub title: String,
    pub content: String,
    pub path: String,
    #[serde(default)]
    pub comments: Vec<Comment>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Comment {
    pub author: String,
    pub text: String,
    pub path: String,
}

impl Into<Bson> for Comment {
    fn into(self) -> Bson {
        Bson::Document(doc! {
            "author": self.author,
            "text": self.text,
            "path": self.path
        })
    }
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
async fn index(data: Data<AppState>) -> impl Responder {
    let database = data.database.lock().await.clone();
    let posts = get_all_posts(database).await;
    let globals = object!(
        {"posts": posts });
    let template = liquid_parse("src/blog_example/web/index.liquid");
    let output = template.render(&globals).unwrap();
    HttpResponse::Ok().content_type("text/html").body(output)
}
#[get("/post/{path}")]
async fn post(data: Data<AppState>, path: web::Path<String>) -> impl Responder {
    let path = path.into_inner();
    let database = data.database.lock().await.clone();
    let globals_post = get_post(database.clone(), path.clone()).await;
    let globals = object!(
        {"post": globals_post, "comments": globals_post.comments});
    let template = liquid_parse("src/blog_example/web/post.liquid");
    let output = template.render(&globals).unwrap();
    HttpResponse::Ok().content_type("text/html").body(output)
}

#[get("/new_post")]
async fn new_post() -> impl Responder {
    let body = read_to_string("src/blog_example/web/create_post.liquid").unwrap();
    HttpResponse::Ok().content_type("text/html").body(body)
}

fn random_path() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    const PASSWORD_LEN: usize = 10;
    let mut rng = rand::thread_rng();

    let password: String = (0..PASSWORD_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    password
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct RequestPost {
    title: String,
    content: String,
}
#[post("/create_post")]
async fn create_post_page(
    data: Data<AppState>,
    post_json: web::Json<RequestPost>,
) -> impl Responder {
    let database = data.database.lock().await.clone();
    let request_post = post_json.into_inner();
    let inserting_post: Post = Post {
        title: request_post.clone().title,
        content: request_post.clone().content,
        path: random_path(),
        comments: vec![],
    };
    create_post(database, inserting_post).await;
    HttpResponse::Ok()
}

#[post("/create_comment")]
async fn create_comment_page(
    data: Data<AppState>,
    comment_json: web::Json<Comment>,
) -> impl Responder {
    println!("{:?}", comment_json);
    let database = data.database.lock().await.clone();
    let request_comment = comment_json.into_inner();
    let comment = Comment {
        author: request_comment.author,
        text: request_comment.text,
        path: request_comment.path,
    };
    let old_post: Post = get_post(database.clone(), comment.path.clone()).await;
    create_comment(database, old_post, comment).await;
    HttpResponse::Ok()
}
#[post("/delete_post")]
async fn delete_post(data: Data<AppState>, post_json: web::Json<String>) -> impl Responder {
    let database = data.database.lock().await.clone();
    let request_post = post_json.into_inner();
    delete_post_query(database, request_post).await;
    HttpResponse::Ok()
}
#[actix_web::main]
pub(crate) async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();
    let db_url = &std::env::var("DATABASE_URL").unwrap();
    let client_options = ClientOptions::parse(db_url).await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("post");
    HttpServer::new(move || {
        let app_state = Data::new(AppState {
            database: Mutex::new(db.clone()),
        });
        App::new()
            .service(index)
            .service(post)
            .service(create_post_page)
            .service(new_post)
            .service(create_comment_page)
            .service(delete_post)
            .app_data(app_state)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
