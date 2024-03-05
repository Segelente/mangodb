use futures::TryStreamExt;
use mongodb::bson::doc;
use mongodb::{Client, ClusterTime, Collection};

use crate::blog_example::blog::{Comment, Post};

pub async fn get_post(client: Client, path: String) -> Post {
    let db = client.database("post");
    let collection = db.collection("post");
    let post = collection
        .find_one(doc! {"path": path.clone()}, None)
        .await
        .unwrap();
    match post {
        Some(post) => post,
        None => panic!("No post found with path {}", path),
    }
}
pub async fn get_all_posts(client: Client) -> Vec<Post> {
    let db = client.database("post");
    let collection = db.collection("post");
    let posts = collection.find(None, None).await.unwrap();
    let vec_posts: Vec<Post> = posts.try_collect().await.unwrap();
    vec_posts
}
pub async fn create_post(client: Client, post: Post) {
    let db = client.database("post");
    let collection = db.collection("post");
    let document = doc! {
        "title": post.title,
        "content": post.content,
        "path": post.path,
    };
    collection.insert_one(document, None).await.unwrap();
}
pub async fn create_comment(client: Client, post: Post, comment: Comment) {
    let db = client.database("post");
    let collection: Collection<Post> = db.collection("post");
    collection
        .find_one_and_update(
            doc! {"path": post.path.clone()},
            doc! {"$push":{"comments": comment} },
            None,
        )
        .await
        .unwrap();
}

pub async fn delete_post_query(client: Client, path: String) {
    let db = client.database("post");
    let collection: Collection<Post> = db.collection("post");
    collection
        .delete_one(doc! {"path": path}, None)
        .await
        .unwrap();
}

#[cfg(test)]
mod tests {
    use mongodb::options::ClientOptions;

    use super::*;

    #[tokio::test]
    async fn test_get_post() {
        dotenvy::dotenv().ok();
        let db_url = &std::env::var("DATABASE_URL").unwrap();
        let client_options = ClientOptions::parse(db_url).await.unwrap();
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
            path: "test1".to_string(),
            comments: vec![],
        };
        let db_url = &std::env::var("DATABASE_URL").unwrap();
        let client_options = ClientOptions::parse(db_url).await.unwrap();
        let client = Client::with_options(client_options).unwrap();
        create_post(client, post).await;
    }
    #[tokio::test]
    async fn test_create_comment() {
        dotenvy::dotenv().ok();
        let comment = Comment {
            text: "Dies ist ein Test".to_string(),
            author: "Bernd".to_string(),
            path: "test1".to_string(),
        };
        let post = Post {
            title: "Dies ist ein Test".to_string(),
            content: "Hallo".to_string(),
            path: "test1".to_string(),
            comments: vec![],
        };
        let db_url = &std::env::var("DATABASE_URL").unwrap();
        let client_options = ClientOptions::parse(db_url).await.unwrap();
        let client = Client::with_options(client_options).unwrap();
        create_comment(client, post, comment).await;
    }
    #[tokio::test]
    async fn test_get_all_posts() {
        dotenvy::dotenv().ok();
        let db_url = &std::env::var("DATABASE_URL").unwrap();
        let client_options = ClientOptions::parse(db_url).await.unwrap();
        let client = Client::with_options(client_options).unwrap();
        let post = get_all_posts(client).await;
        println!("{:?}", post)
    }
}
