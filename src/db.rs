use mongodb::{bson::{doc, Document, RawDocumentBuf}, options::{ClientOptions, FindOptions}, Client, Database};


async fn connect_to_db() -> Database {
    let mut client_options = ClientOptions::parse("<key>").await.unwrap();
    client_options.app_name = Some("TacoDB".to_string());
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("TacoDatabase");
    db
}

pub async fn post_project_to_db(name: String, version: String, authors: Vec<String>, repo: String) -> mongodb::error::Result<()> {
    let db = connect_to_db().await;
    let collection = db.collection::<Document>("UnverifiedProjects");
    
    let doc = doc! { "name": name, "version": version, "authors": authors, "repo": repo };

    collection.insert_one(doc, None).await?;

    Ok(())
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
        docs.push(cursor.current().to_raw_document_buf());
    }

    docs
}