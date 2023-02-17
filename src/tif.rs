use crate::db::*;
use std::io::{Write};
use std::fs::{File, create_dir};
use mongodb::bson::{doc};

//-----------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ProjectData {
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub repo: String,
    pub pid: String
}

impl ProjectData {
    fn new(name: String, version: String, authors: Vec<String>, repo: String, pid: String) -> ProjectData {
        ProjectData {
            name: name,
            version: version,
            authors: authors,
            repo: repo,
            pid: generate_pid() // change to find pid in db
        }
    }
}

// add pid to project data
fn load_toml_project_data(path: String) -> ProjectData {
    use toml::{Table};
    
    let taco_file = format!("{}/taco.toml", path);
    let file_contents = std::fs::read_to_string(taco_file).unwrap();

    let x: Table = file_contents.parse::<Table>().unwrap();
    
    ProjectData::new(
        x["package"]["name"].as_str().unwrap().to_string(),
        x["package"]["version"].as_str().unwrap().to_string(),
        x["package"]["authors"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect(),
        x["package"]["repo"].as_str().unwrap().to_string(),
        get_pid(path)
    )
}

fn generate_pid() -> String {
    use rand::{Rng};
    let mut rng = rand::thread_rng();
    let mut pid = String::new();
    for _ in 0..20 {
        pid.push(rng.gen_range(0..=9).to_string().chars().next().unwrap());
    }
    pid
}

fn get_pid(path: String) -> String {
    use toml::{Table};


    let taco_file = format!("{}/taco.toml", path);
    let file_contents = std::fs::read_to_string(taco_file).unwrap();

    return file_contents.parse::<Table>().unwrap()["pid"].as_str().unwrap().to_string();
}

//-----------------------------------------------------------------------------------------------

const CONFIG: &[u8] = b"[package]
name = \"main\"
version = \"0.0.1\"
authors = [\"\"]
clean = [\"\"]

[dependencies]
";
const CPP: &[u8] = b"#include <iostream>
        
int main() {
    std::cout << \"Hello, World!\" << std::endl;
    return 0;
}";

#[derive(Debug, Clone)]
pub struct Subcommand {
    pub args: Vec<String>,
    pub dir_path: String,
    pub exe_path: String,
}

//-----------------------------------------------------------------------------------------------

pub async fn init(command: &Subcommand) -> i32 {
    // Make the taco.toml file
    let mut file = File::create(format!("{}/taco.toml", command.dir_path)).unwrap();
    file.write_all(CONFIG).unwrap();

    // Make the taco.lock file and th pid
    file = File::create(format!("{}/taco.lock", command.dir_path)).unwrap();
    let mut pid = generate_pid();
    while pid_exists_check(&pid).await == 1 {
        pid = generate_pid();
    }
    file.write_all(format!("pid = \"{}\"", pid).as_bytes()).unwrap();

    // Make the source directory
    create_dir(format!("{}/source", command.dir_path))
    .expect("Error: Could not create source directory.");

    // Make the main.cpp file
    file = File::create(format!("{}/source/main.cpp", command.dir_path)).unwrap();
    file.write_all(CPP).unwrap();

    0
}

pub async fn new(command: &Subcommand) -> i32 {
    // Get the name of the project
    let name = command.args.get(2)
    .expect("Error: No name given.");
    let path = format!("{}/{}", command.dir_path, name);

    // Make the project directory
    create_dir(&path).expect("Error: Could not create directory.");

    // Make the taco.toml file
    let mut file = File::create(format!("{path}/taco.toml")).unwrap();
    file.write_all(CONFIG).unwrap();

    // Make the taco.lock file
    file = File::create(format!("{path}/taco.lock")).unwrap();
    let mut pid = generate_pid();
    while pid_exists_check(&pid).await == 1 {
        pid = generate_pid();
    }
    file.write_all(format!("pid = \"{}\"", pid).as_bytes()).unwrap();

    // Make the source directory
    create_dir(format!("{path}/source"))
    .expect("Error: Could not create source directory.");

    // Make the main.cpp file
    file = File::create(format!("{path}/source/main.cpp")).unwrap();
    file.write_all(CPP).unwrap();

    0
}

pub fn clean(command: &Subcommand) -> i32 {
    use toml::{Value, from_str};
    
    let taco = format!("{}/taco.toml", command.dir_path);
    let file = std::fs::read_to_string(taco).unwrap();

    let x: Value = from_str(&file).unwrap();

    let clean = x.get("package").unwrap().get("clean").unwrap().as_array().unwrap();

    for i in clean {
        let mut files = std::fs::read_dir(format!("{}/source", command.dir_path)).unwrap();
        let i = i.as_str().unwrap();
        
        while let Some(Ok(file)) = files.next() {
            let file_name = file.file_name();
            let Some(file_name) = file_name.to_str() else { continue; };
            let file_path = file.path();

            if i.starts_with("*.") && file_name.ends_with(
                &i[2..]
            ) {
                std::fs::remove_file(file_path).unwrap();
            }
            else if i.ends_with(".*") && file_name.starts_with(
                i.trim_end_matches(".*")
            ) {
                std::fs::remove_file(file_path).unwrap();
            }
            else if i == file_name {
                std::fs::remove_file(file_path).unwrap();
            }
        }
    }

    0
}

pub fn add(command: &Subcommand) -> i32 {
    0
}

pub fn remove(command: &Subcommand) -> i32 {
    0
}

pub async fn wrap(command: &Subcommand) -> i32 {
    let project = load_toml_project_data(format!("{}", command.dir_path));

    if pid_exists_check(&project.pid).await == 0 {
        match post_project_to_db(project).await {
            Ok(_) => (),
            Err(x) => panic!("Error: {}", x)
        };
    }
    else {
        update_project_in_db(project).await.expect("Error: Could not update project in database.");
    }

    0
}

pub async fn search(command: &Subcommand) ->i32 {
    let remaining_args = command.args.clone().split_off(2).join(" ");
    let docs = get_project_from_db(doc! { "name": remaining_args }).await;

    for i in docs {
        println!("{}: {}", match i.get("name") {
            Ok(Some(x)) => x.as_str().unwrap(),
            _ => panic!("Error: Could not get name.")
        },
        match i.get("version") {
            Ok(Some(x)) => x.as_str().unwrap(),
            _ => panic!("Error: Could not get version.")
        });
    }
    0
}

pub fn run(command: &Subcommand) ->i32 { // Change when they are included (as opposed to the command)
    use std::process::Command;
    if command.args.len() < 3 {
        let output = Command::new("g++")
            .args([
                &format!("{}/source/main.cpp", command.dir_path),
                "-o",
                &format!("{}/source/main", command.dir_path)
            ])
            .args([
                &format!("-I{}/include", command.exe_path),
                &format!("-L{}/lib", command.exe_path),
            ])
            .output().expect("Error: Could not compile.").stdout;
        println!("{}", String::from_utf8_lossy(&output));

        let output = Command::new(format!("{}/source/main.exe", command.dir_path))
            .output().expect("Error: Could not run.").stdout;
        println!("{}", String::from_utf8_lossy(&output));
    }
    else {
        let name: &String = command.args.get(2).unwrap();

        let remaining_args = command.args.clone().split_off(3);

        let final_path = command.dir_path.clone() + "/source/" + name.trim_end_matches(".cpp");

        for i in 0..remaining_args.len() {
            if remaining_args[i].starts_with("-o") {
                panic!("Error: Cannot specify name of output file.");
            }
        }

        let output = Command::new("g++")
            .args([&format!("{}/source/{name}", command.dir_path), "-o", &final_path])
            .args(&remaining_args)
            .args([
                &format!("-I{}/include", command.exe_path),
                &format!("-L{}/lib", command.exe_path),
            ])
            .output().expect("Error: Could not compile.").stdout;
        println!("{}", String::from_utf8_lossy(&output));

        let output = Command::new(&final_path)
            .output().expect("Error: Could not run.").stdout;
        
        println!("{}", String::from_utf8_lossy(&output));
    };

    0
}

pub fn help() -> i32 {
    println!(
        "How to Use:
        taco <command> [args]
        
        <commands>:
        Init: Creates a new taco workplace in your current directory.
        New [Name]: Creates a new taco workplace in \"current_dir()/Name\".
        Clean: Removes all files specified in the \"clean\" array in the \"taco.toml\" file.
        (can use wildcards [eg. *.o, main.*])
        
        Add [name]: Searches the package database for \"Name\" and adds it to your dependancies.
        Remove [Name]: Searches your dependancies for \"Name\" and removes it.
        
        Run: Compiles and runs the taco workplace.
        Run [Name]: Compiles and runs the file \"Name\".
        
        Help: Displays the Help Message.
        "
    );
    0
}
