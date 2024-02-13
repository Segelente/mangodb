use mongodb::Client;

pub fn get_post(client: Client, id: i32) {
    let db = client.database("post");
    db.collection("post");
}