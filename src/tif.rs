#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_macros)]
#![allow(dead_code)]

use crate::db::{post_project_to_db, get_project_from_db};
use std::io::{self, Write, Read};
use std::fs::{File, create_dir};

macro_rules! matcher {
    ($match:expr, $ok:expr, $err:expr) => {
        match $match {
            Ok(_) => $ok,
            Err(_) => $err
        }
    };
}

const CONFIG: &[u8] = b"[package]
name = \"taco\"
version = \"0.1.0\"
authors = [\"Taco Team\"]
clean = []

[dependencies]
";
const CPP: &[u8] = b"#include <iostream>
        
int main() {
    std::cout << \"Hello, World!\" << std::endl;
    return 0;
}";

#[derive(Debug, Clone)]
pub struct Subcommand { // like methab
    pub args: Vec<String>,
    pub path: String
}

pub fn init(command: Subcommand) -> i32 {
    let path = command.path;

    let mut file = File::create(path.clone() + "/taco.toml").unwrap();
    file.write_all(CONFIG).unwrap();

    matcher!( create_dir(path.clone() + "/source"), 
    (), panic!("Error: Could not create source directory."));

    file = File::create(path.clone() + "/source/main.cpp").unwrap();
    file.write_all(CPP).unwrap();
    0
}

pub fn new(command: Subcommand) -> i32 {
    let path = command.path + "/" + command.args.get(2).expect("Error: No name given.");

    create_dir(path.clone()).expect("Error: Could not create directory.");

    let mut file = File::create(path.clone() + "/" + "taco.toml").unwrap();
    file.write_all(CONFIG).unwrap();

    create_dir(path.clone() + "/source").expect("Error: Could not create sourc directory.");
    file = File::create(path.clone() + "/source/main.cpp").unwrap();
    file.write_all(CPP).unwrap();
    0
}

pub fn clean(command: Subcommand) -> i32 {
    use toml::{Value, from_str};
    
    let taco = command.path.clone() + "/taco.toml";
    let file = std::fs::read_to_string(taco).unwrap();

    let x: Value = from_str(&file).unwrap();

    let clean = x.get("package").unwrap().get("clean").unwrap().as_array().unwrap();

    for i in clean {
        let mut files = std::fs::read_dir(command.path.clone() + "/source").unwrap();
        let i = i.as_str().unwrap();
        
        while let Some(Ok(file)) = files.next() {
            let file_name = file.file_name();
            let Some(file_name) = file_name.to_str() else { continue; };
            let file_path = file.path();

            if i.starts_with("*.") && file_name.ends_with(&i[2..]) {
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

pub fn add(command: Subcommand) -> i32 {
    0
}

pub fn remove(command: Subcommand) -> i32 {
    0
}

pub async fn wrap(command: Subcommand) -> i32 {
    use toml::{Value, from_str};
    
    let taco = command.path.clone() + "/taco.toml";
    let file = std::fs::read_to_string(taco).unwrap();

    let x: Value = from_str(&file).unwrap();

    let package = x.get("package").unwrap();

    match post_project_to_db(
        package.get("name").unwrap().as_str().unwrap().to_string(),
        package.get("version").unwrap().as_str().unwrap().to_string(),
        package.get("authors").unwrap().as_array().unwrap().get(0).unwrap().as_str().unwrap().to_string(),
        "a".to_string()
    ).await {
        Ok(_) => (),
        Err(_) => panic!("Error: Could not post project to database.")
    };
    
    0
}

pub fn run(command: Subcommand) ->i32 {
    use std::process::Command;
    if command.args.len() < 3 {
        let output = Command::new("g++")
            .arg(command.path.clone() + "/source/main.cpp")
            .arg("-o")
            .arg(command.path.clone() + "/source/main")
            .output()
            .expect("Error: Could not compile.").stdout;
        println!("{}", String::from_utf8_lossy(&output));

        let output = Command::new(command.path.clone() + "/source/main.exe")
            .output()
            .expect("Error: Could not run.")
            .stdout;
    
        println!("{}", String::from_utf8_lossy(&output));
    }
    else {
        let name: &std::string::String = command.args.get(2).unwrap();

        let remaining_args = command.args.clone().split_off(3);

        let final_path = command.path.clone() + "/source/" + name.trim_end_matches(".cpp");

        for i in 0..remaining_args.len() {
            if remaining_args[i].starts_with("-o") {
                panic!("Error: Cannot specify name of output file.");
            }
        }

        let output = Command::new("g++")
            .arg(command.path.clone() + "/source/" + name)
            .args(&remaining_args)
            .arg("-o")
            .arg(&final_path)
            .output()
            .expect("Error: Could not compile.").stdout;

        println!("{}", String::from_utf8_lossy(&output));

        let output = Command::new(&final_path)
            .output()
            .expect("Error: Could not run.")
            .stdout;
        
        println!("{}", String::from_utf8_lossy(&output));
    };

    0
}

pub fn help(command: Subcommand) -> i32 {
    println!(
        "
        How to Use:
        taco <command> [args]
        
        <commands>:
        Init: Creates a new taco workplace in your current directory.
        New [Name]: Creates a new taco workplace in \"current_dir()/Name\".
        Clean: Cleans taco workplace (?)
        
        Add [name]: Searches the package database for \"Name\" and adds it to your dependancies.
        Remove [Name]: Searches your dependancies for \"Name\" and removes it.
        
        Run: Compiles and runs the taco workplace.
        Run [Name]: Compiles and runs the file \"Name\".
        
        Help: Displays the Help Message.
        "
    );
    0
}