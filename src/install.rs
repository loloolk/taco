#![allow(unused_imports)]
use std::io::{self, Write};
use std::fs::{File, create_dir};

macro_rules! matcher {
    ($match:expr, $ok:expr, $err:expr) => {
        match $match {
            Ok(_) => $ok,
            Err(_) => $err
        }
    };
}


fn install() -> i32 {
    use std::process::Command;

    let this_dir = std::env::current_exe().unwrap().parent().unwrap().to_str().unwrap().to_string();
    
    let current_path = std::env::var("PATH").unwrap();

    Command::new("cmd")
        .args(&["/C", "setx", "PATH", &(current_path + ";" + &this_dir)])
        .output()
        .expect("failed to execute process");


    create_dir(format!("{}/bin", this_dir)).expect("Error: Could not create bin directory.");
    create_dir(format!("{}/lib", this_dir)).expect("Error: Could not create lib directory.");
    create_dir(format!("{}/include", this_dir)).expect("Error: Could not create include directory.");
    
    0
}

fn main() {
    install();
}
