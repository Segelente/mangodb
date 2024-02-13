//! Webpage for blog posts.
use std::fs::read_to_string;
use std::path::Path;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use liquid::{object, Template};
use serde::Serialize;


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
async fn post() -> impl Responder{
    let globals = object!({
        "post": Post {
            title: "Hello, world!".to_string(),
            content: "This is a blog post".to_string(),
            path: "/post/1".to_string(),
        }
    });
    let template = liquid_parse("src/web/post.liquid");
    let output = template.render(&globals).unwrap();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(output)
}

#[actix_web::main]
pub(crate) async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(post)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}