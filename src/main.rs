#![feature(proc_macro_hygiene, decl_macro)]

use rocket::{
    request, response, get,
    config::{Config, Environment},
};
use rocket_contrib::serve::StaticFiles;
use std::{env};

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() < 2 {
        eprintln!("Usage: share_it <resource> <shared_name>");
        return;
    }

    let config = Config::build(Environment::Production)
        .address("192.168.56.1")
        .port(80)
        .finalize()
        .unwrap();
    
    rocket::custom(config)
        .mount(&format!("/{}", args[1]), StaticFiles::from(args[0].clone()))
        .launch();
}
