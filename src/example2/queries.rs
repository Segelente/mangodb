use futures::{StreamExt, TryStreamExt};
use mongodb::bson::doc;
use mongodb::Client;
use serde::de::Error;
use crate::example2::blog::{Comment, Post};
pub async fn get_post(client: Client, path: String) -> Post {
    let db = client.database("post");
    let collection = db.collection("post");
    let post = collection.find_one(doc! {"path": path.clone()}, None).await.unwrap();
    match post {
        Some(post) => post,
        None => panic!("No post found with path {}", path)
    }
}
pub async fn get_all_posts(client: Client) -> Vec<Post> {
    let db = client.database("post");
    let collection = db.collection("post");
    let posts = collection.find(None, None).await.unwrap();
    let vec_posts: Vec<Post> = posts.try_collect().await.unwrap();
    vec_posts
}
pub async fn get_comment(client: Client, path: String) -> Vec<Comment> {
    let db = client.database("comment");
    let collection = db.collection("comment");
    let comments = collection.find(doc! {"path": path.clone()}, None).await.unwrap();
    let vec_comments: Vec<Comment> = comments.try_collect().await.unwrap();
    vec_comments
}
pub async fn create_post(client: Client, post: Post) {
    let db = client.database("post");
    let collection = db.collection("post");
    collection.insert_one(post, None).await.unwrap();
}
pub async fn create_comment(client: Client, comment: Comment) {
    let db = client.database("comment");
    let collection = db.collection("comment");
    collection.insert_one(comment, None).await.unwrap();
}

#[cfg(test)]
mod tests{
    use mongodb::options::ClientOptions;
    use super::*;

    #[tokio::test]
    async fn test_get_post(){
        dotenvy::dotenv().ok();
        let db_url = &std::env::var("DATABASE_URL").unwrap();
        let  client_options = ClientOptions::parse(db_url).await.unwrap();
        let client = Client::with_options(client_options).unwrap();
        let post = get_post(client, "Hello, world!".to_string()).await;
        assert_eq!(post.title, "Hello, world!");
    }

    #[tokio::test]
    async fn test_create_post() {
        dotenvy::dotenv().ok();
        let post = Post {
            title: "Dies ist ein Test".to_string(),
            content: "Hallo".to_string(),
            path: "test1".to_string()
        };
        let db_url = &std::env::var("DATABASE_URL").unwrap();
        let  client_options = ClientOptions::parse(db_url).await.unwrap();
        let client = Client::with_options(client_options).unwrap();
        create_post(client, post).await;
    }
    #[tokio::test]
    async fn test_create_comment() {
        dotenvy::dotenv().ok();
        let comment = Comment {
            text: "Dies ist ein Test".to_string(),
            author: "Bernd".to_string(),
            path: "test1".to_string()
        };
        let db_url = &std::env::var("DATABASE_URL").unwrap();
        let  client_options = ClientOptions::parse(db_url).await.unwrap();
        let client = Client::with_options(client_options).unwrap();
        create_comment(client, comment).await;
    }
    #[tokio::test]
    async fn test_get_comments(){
        dotenvy::dotenv().ok();
        let db_url = &std::env::var("DATABASE_URL").unwrap();
        let  client_options = ClientOptions::parse(db_url).await.unwrap();
        let client = Client::with_options(client_options).unwrap();
        let comment = get_comment(client, "test1".to_string()).await;
        assert_eq!("Bernd", comment[0].author)
    }
    #[tokio::test]
    async fn test_get_all_posts(){
        dotenvy::dotenv().ok();
        let db_url = &std::env::var("DATABASE_URL").unwrap();
        let  client_options = ClientOptions::parse(db_url).await.unwrap();
        let client = Client::with_options(client_options).unwrap();
        let post = get_all_posts(client).await;
        println!("{:?}", post)
    }

}