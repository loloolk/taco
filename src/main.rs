#![allow(unused_variables)]
mod tif;
mod db;
use tif::Subcommand;

#[tokio::main]
async fn main() {
    {
        use std::env::args;

        let args: Vec<String> = args().collect();
        let path = std::env::current_dir().unwrap().to_str().unwrap().to_string();

        let command: Subcommand = Subcommand {
            args: args,
            path: path
        };

        let error_code: i32 = {
            if command.args.len() == 1 {
                dbg!("help");
                tif::help(command.clone())
            }  
            else { 
                match command.args[1].to_lowercase().as_str() {
                    "init" => tif::init(command.clone()),
                    "new" => tif::new(command.clone()),
                    "clean" => tif::clean(command.clone()),
                    "wrap" => tif::wrap(command.clone()).await,
                    "add" => tif::add(command.clone()),
                    "remove" => tif::remove(command.clone()),
                    "run" => tif::run(command.clone()),
                    "help" => tif::help(command.clone()),
                    _ => tif::help(command.clone()),
                }
            }
        };
        
    }
}