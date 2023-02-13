#![allow(dead_code)]
use mongodb::{bson::{doc, Document, RawDocumentBuf}, options::{ClientOptions, FindOptions}, Client, Database};
use crate::tif::ProjectData;

async fn connect_to_db() -> Database {
    let mut client_options = ClientOptions::parse(
        std::fs::read_to_string("../db.txt").unwrap().trim()
    ).await.unwrap();
    client_options.app_name = Some("TacoDB".to_string());
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("TacoDatabase");
    db
}

pub async fn post_project_to_db(project: ProjectData) -> mongodb::error::Result<()> {
    let db = connect_to_db().await;
    let collection = db.collection::<Document>("UnverifiedProjects");
    
    let doc = doc! { "name": project.name, "version": project.version, "authors": project.authors, "repo": project.repo };

    collection.insert_one(doc, None).await.map(|_|())
}

pub async fn get_project_from_db(name: String) -> Vec<RawDocumentBuf> {
    let db = connect_to_db().await;
    
    let collection = db.collection::<Document>("UnverifiedProjects");

    let options = FindOptions::builder().limit(10).build();

    let mut cursor = match collection.find(doc! { "name": name }, options).await {
        Ok(cursor) => cursor,
        Err(x) => panic!("{}", x)
    };

    let mut docs = Vec::new();

    while cursor.advance().await.unwrap() {
        docs.push(cursor.current().to_raw_document_buf());
    }

    docs
}

pub async fn get_exact_copy_from_db(project: ProjectData) -> RawDocumentBuf {
    let db = connect_to_db().await;
    
    let collection = db.collection::<Document>("UnverifiedProjects");

    let options = FindOptions::builder().limit(10).build();

    let cursor = 
    match collection.find(
        doc! { 
            "name": project.name, 
            "version": project.version, 
            "authors": project.authors, 
            "repo": project.repo 
        }, options).await {
        Ok(cursor) => cursor,
        Err(x) => panic!("{}", x)
    };

    cursor.current().to_raw_document_buf()
}

pub async fn update_uid_database(uid: String) -> i32 {
    let db = connect_to_db().await;
    let collection = db.collection::<Document>("UID");

    // Check if UID is already in database, if it is, return 1
    let options = FindOptions::builder().limit(1).build();
    let mut cursor = match collection.find(doc! { "uid": &uid }, options).await {
        Ok(cursor) => cursor,
        Err(x) => panic!("{}", x)
    };
    if cursor.advance().await.unwrap() {
        return 1;
    }
    
    let doc = doc! { "uid": uid };

    collection.insert_one(doc, None).await.map(|_|0).unwrap();

    0
}
