#![allow(dead_code)]
use mongodb::{bson::{doc, Document, RawDocumentBuf}, options::{ClientOptions, FindOptions}, Client, Database};
use crate::tif::ProjectData;

async fn connect_to_db() -> Database {
    let mut client_options = ClientOptions::parse(
        std::fs::read_to_string("C:\\Users\\marcb_xqarsni\\Desktop\\taco\\db.txt").unwrap().trim() // fix so tht it's global
    ).await.unwrap();
    client_options.app_name = Some("TacoDB".to_string());
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("TacoDatabase");
    db
}

pub async fn post_project_to_db(project: ProjectData) -> mongodb::error::Result<()> {
    let db = connect_to_db().await;
    let collection = db.collection::<Document>("UnverifiedProjects");
    
    let doc = doc! { 
        "name": project.name, 
        "version": project.version, 
        "authors": project.authors, 
        "repo": project.repo,
        "pid": project.pid
    };

    collection.insert_one(doc, None).await.map(|_|())
}

pub async fn get_project_from_db(query: Document) -> Vec<RawDocumentBuf> {
    let db = connect_to_db().await;
    
    let collection = db.collection::<Document>("UnverifiedProjects");

    let options = FindOptions::builder().limit(10).build();

    let mut cursor = match collection.find(query, options).await {
        Ok(cursor) => cursor,
        Err(x) => panic!("{}", x)
    };

    let mut docs = Vec::new();

    while cursor.advance().await.unwrap() {
        docs.push(cursor.current().to_raw_document_buf());
    }

    docs
}

pub async fn get_pid_copy_from_db(project: ProjectData) -> RawDocumentBuf {
    return get_project_from_db(doc! { "pid": project.pid }).await[0].clone();
}

pub async fn pid_exists_check(pid: &String) -> i32 {
    if get_project_from_db(doc! { "pid": pid }).await.len() == 0 {
        return 0;
    }

    1
}

pub async fn update_project_in_db(project: ProjectData) -> mongodb::error::Result<()> { // not sure if works
    let db = connect_to_db().await;
    let collection = db.collection::<Document>("UnverifiedProjects");

    let doc = doc! { 
        "name": project.name, 
        "version": project.version, 
        "authors": project.authors, 
        "repo": project.repo,
        "pid": &project.pid
    };

    collection.update_one(doc! { "pid": &project.pid }, doc, None).await.map(|_|())
}
