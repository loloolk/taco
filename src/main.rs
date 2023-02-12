#![allow(unused_variables)]
mod tif;
mod db;

#[derive(Debug, Clone)]
pub struct Subcommand {
    pub args: Vec<String>,
    pub path: String
}

#[tokio::main]
async fn main() {
    use std::env::args;

    let args: Vec<String> = args().collect();
    let dir_path = std::env::current_dir().unwrap().to_str().unwrap().to_string();
    let exe_path = std::env::current_exe().unwrap().to_str().unwrap().to_string();

    let command: Subcommand = Subcommand {
        args: args,
        dir_path: dir_path,
        exe_path: exe_path
    };

    let error_code: i32 = {
        if command.args.len() == 1 {
            tif::help()
        }  
        else { 
            match command.args[1].to_lowercase().as_str() {
                "init" => tif::init(&command),
                "new" => tif::new(&command),
                "clean" => tif::clean(&command),
                "wrap" => tif::wrap(&command).await,
                "search" => tif::search(&command).await,
                "add" => tif::add(&command),
                "remove" => tif::remove(&command),
                "run" => tif::run(&command),
                "help" => tif::help(),
                _ => tif::help(),
            }
        }
    };
}