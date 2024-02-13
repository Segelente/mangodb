use mongodb::bson::doc;
use mongodb::Client;
use crate::example2::blog::Post;

pub async fn get_post(client: Client, path: String) -> Post {
    let db = client.database("post");
    let collection = db.collection("post");
    let post = collection.find_one(doc! {"path": path.clone()}, None).await.unwrap();
    match post {
        Some(post) => post,
        None => panic!("No post found with path {}", path)
    }
}
pub async fn create_post(client: Client, post: Post) {
    let db = client.database("post");
    let collection = db.collection("post");
    collection.insert_one(post, None).await.unwrap();
}

#[cfg(test)]
mod tests{
    use super::*;

    #[tokio::test]
    async fn test_get_post(){
        let client = Client::with_uri_str("mongodb://localhost:27017").await.unwrap();
        let post = get_post(client, "Hello, world!".to_string()).await;
        assert_eq!(post.title, "Hello, world!");
    }
}