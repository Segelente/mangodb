use mongodb::{Client, options::ClientOptions};
use mongodb::bson::{doc, Document};
#[tokio::main]
async fn main() {
    let db_url = &std::env::var("DATABASE_URL").unwrap();
    let  client_options = ClientOptions::parse(db_url).await.unwrap();

    let client = Client::with_options(client_options).unwrap();

    let db = client.database("mydb");

    let collection = db.collection::<Document>("fruits");

    let fruitys = vec![
        doc! { "name": "apple", "color": "red" },
        doc! { "name": "banana", "color": "yellow" },
        doc! { "name": "cherry", "color": "red" },
        doc! {"name": "mango", "color": "orange-green" },
    ];

    collection.insert_many(fruitys, None).await.unwrap();
    println!("{:?}",db.list_collection_names(None).await.unwrap());
}
