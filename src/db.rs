use mongodb::{bson::{doc, Document, RawDocumentBuf}, options::{ClientOptions, FindOptions}, Client, Database, Cursor};

async fn connect_to_db() -> Database {
    let mut client_options = ClientOptions::parse(
        std::fs::read_to_string("../db.txt").unwrap().trim()
    ).await.unwrap();
    client_options.app_name = Some("TacoDB".to_string());
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("TacoDatabase");
    db
}

pub async fn post_project_to_db(name: String, version: String, authors: Vec<String>, repo: String) -> mongodb::error::Result<()> {
    let db = connect_to_db().await;
    let collection = db.collection::<Document>("UnverifiedProjects");
    
    let doc = doc! { "name": name, "version": version, "authors": authors, "repo": repo };

    collection.insert_one(doc, None).await.map(|_|())
}

pub async fn get_project_from_db(name: String) -> Vec<RawDocumentBuf> {
    let db = connect_to_db().await;
    
    let collection = db.collection::<Document>("UnverifiedProjects");

    let find_options = FindOptions::builder().sort(doc! { "name": 1 }).build();
    
    let mut cursor = match collection.find(doc! { "name": name }, find_options).await {
        Ok(cursor) => cursor,
        Err(x) => panic!("{}", x)
    };

    let mut docs = Vec::new();

    while cursor.advance().await.unwrap() {
        docs.push(cursor.current());
    }

    docs
}
