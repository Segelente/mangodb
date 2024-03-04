//! Webpage for blog posts.
use std::fs::read_to_string;
use std::path::Path;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web::cookie::time::macros::date;
use actix_web::web::{Data, patch};
use liquid::{object, ObjectView, Template, ValueView};
use mongodb::Client;
use dotenvy;
use mongodb::bson::{Bson, doc};
use tracing::info;
use mongodb::options::ClientOptions;
use serde::{Deserialize, Serialize};
use rand::Rng;
use tokio::sync::Mutex;
use crate::example2::queries::{create_comment, create_post, get_all_posts, get_post};

pub(crate) struct AppState {
    client: Mutex<Client>,
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
async fn index(data: Data<AppState>) -> impl Responder{
    let client = data.client.lock().await.clone();
    let posts = get_all_posts(client).await;
    let globals = object!(
        {"posts": posts });
    let template = liquid_parse("src/example2/web/index.liquid");
    println!("{:?}", globals);
    let output = template.render(&globals).unwrap();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(output)
}
#[get("/post/{path}")]
async fn post(data: Data<AppState>, path: web::Path<String>) -> impl Responder{
    let path = path.into_inner();
    let client = data.client.lock().await.clone();
    info!("Getting post with id {}", path.clone());
    let globals_post = get_post(client.clone(), path.clone()).await;
    let globals = object!(
        {"post": globals_post, "comments": globals_post.comments});
    let template = liquid_parse("src/example2/web/post.liquid");
    println!("{:?}", globals);
    let output = template.render(&globals).unwrap();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(output)
}

#[get("/new_post")]
async fn new_post() -> impl Responder {
    let body = read_to_string("src/example2/web/create_post.liquid").unwrap();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(body)
}

fn random_path() -> String{
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
async fn create_post_page(data: Data<AppState>, mut post_json: web::Json<RequestPost>) -> impl Responder {
    println!("{:?}", post_json);
    let client = data.client.lock().await.clone();
    let request_post = post_json.into_inner();
    let inserting_post: Post = Post {
        title: request_post.clone().title,
        content: request_post.clone().content,
        path: random_path(),
        comments: vec![]
    };
    create_post(client, inserting_post).await;
    HttpResponse::Ok()
}

#[post("/create_comment")]
async fn create_comment_page(data: Data<AppState>, comment_json: web::Json<Comment>) -> impl Responder{
    println!("{:?}", comment_json);
    let client = data.client.lock().await.clone();
    let request_comment = comment_json.into_inner();
    let comment = Comment {
        author: request_comment.author,
        text: request_comment.text,
        path: request_comment.path
    };
    let mut old_post: Post = get_post(client.clone(), comment.path.clone()).await;

    let inserting_post = old_post.clone();
    create_comment(client, old_post, comment).await;
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
            .service(new_post)
            .service(create_comment_page)
            .app_data(app_state)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}