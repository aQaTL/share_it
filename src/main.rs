#![feature(proc_macro_hygiene, decl_macro)]

use rocket::{
	get, handler,
	request::{Request, State},
	response::content,
	config::{Config, Environment},
};
use rocket_contrib::{
	serve::StaticFiles,
};
use std::{
	path::PathBuf,
	fs,
};
use serde_derive::Serialize;

fn clap_app() -> clap::App<'static, 'static> {
	use clap::*;
	App::new(crate_name!())
		.version(crate_version!())
		.about(crate_description!())
		.arg(Arg::with_name("resource")
			.required(true)
			.multiple(true)
			.takes_value(true)
			.index(1)
			.default_value("."))
		.arg(Arg::with_name("name")
			.help("e.g. `--name foo` will result in sharing the resource on `/foo`")
			.short("n")
			.takes_value(true)
			.empty_values(false)
			.default_value("s"))
		.arg(Arg::with_name("address")
			.help("ip of the network interface on which the application will serve")
			.long("address")
			.takes_value(true)
			.default_value("127.0.0.1"))
		.arg(Arg::with_name("port")
			.help("port on which the application will listen")
			.short("p")
			.long("port")
			.takes_value(true)
			.default_value("80"))
}

fn main() {
	let app = clap_app().get_matches();
	let port = app.value_of("port").unwrap();
	let port = match port.parse::<u16>() {
		Ok(port) => port,
		Err(_) => {
			graceful_exit(&format!(
				"Invalid value, could not parse `{}` as a port number (0 - 65535)", port));
			0
		}
	};

	let address = app.value_of("address").unwrap();
	let name = app.value_of("name").unwrap();

	let resource_str = app.value_of("resource").unwrap();
	let resource = PathBuf::from(resource_str);
	if !resource.exists() {
		graceful_exit(&format!("{} not found", resource_str));
	}
	let mut resource = resource.canonicalize().unwrap();
	if resource.is_file() {
		resource.pop();
	}

	let resource_dir = match fs::read_dir(resource.clone()) {
		Ok(dir) => dir.collect::<Vec<_>>(),
		Err(err) => {
			graceful_exit(&format!("error reading {}: {}",
								   resource.display(), err.to_string()));
			unreachable!()
		}
	};

	println!("Sharing {} on {}:{}/{}/",
			 resource.file_name().unwrap().to_string_lossy(), address, port, name);

	let config = Config::build(Environment::Production)
		.address(address)
		.port(port)
		.finalize()
		.unwrap();

	let resource_dir = resource_dir
		.into_iter()
		.filter_map(|e| e.ok())
		.map(|e| e.file_name().into_string().unwrap())
		.collect::<Vec<_>>();

	for e in &resource_dir {
		println!("{}", e);
	}

	let mut static_files: Vec<rocket::Route> = StaticFiles::new(
		resource, rocket_contrib::serve::Options::DotFiles).into();
	let get_dir_ls_route = rocket::Route::new(rocket::http::Method::Get, "/", get_dir_ls);
	static_files.push(get_dir_ls_route.clone());

	rocket::custom(config)
		.mount(&format!("/{}", name), static_files)
		.mount("/index", vec![get_dir_ls_route])
		.mount("/", rocket::routes![index])
//		.mount("/frontend", ) //TODO serving frontend files included in binary (include! macro)
		.manage(ResourceDir(resource_dir)) //TODO handle dir content changes
		.launch();
}

#[derive(Serialize)]
struct ResourceDir(Vec<String>);

#[get("/")]
fn index() -> content::Html<&'static str> {
	content::Html(include_str!("../frontend/index.html"))
}

fn get_dir_ls<'r>(req: &'r Request, _: rocket::Data) -> handler::Outcome<'r> {
	let resource_dir = req.guard::<State<ResourceDir>>().unwrap().inner();
	rocket::Outcome::from(req, rocket::response::content::Json(serde_json::to_string(resource_dir)))
}


fn graceful_exit(err: &str) {
	eprintln!("{}", err);
	std::process::exit(1);
}
