#![feature(proc_macro_hygiene, decl_macro, uniform_paths)]

use rocket::{
	get,
	response::{content},
	http,
	config::{Config, Environment},
};
use std::{
	path::PathBuf,
	fs,
};
use serde::Serialize;

mod frontend;
mod staticfiles;

use crate::{frontend::*, staticfiles::*};

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
			.help("e.g. `--name foo` will result in sharing the resource on `/s/foo`")
			.short("n")
			.takes_value(true)
			.empty_values(true)
			.default_value(""))
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

	let mut routes = rocket::routes![index, serve_frontend];
	routes.append(&mut StaticFilesBrowser::new(resource).into());

	rocket::custom(config)
		.mount("/", routes)
		.launch();
}

#[derive(Serialize)]
struct ResourceDir(Vec<String>);

#[get("/")]
fn index() -> content::Html<&'static str> {
	content::Html(FRONTEND_FILES.get("index.html").unwrap())
}

#[get("/<resource..>", rank=2)]
fn serve_frontend(resource: PathBuf) -> Option<content::Content<&'static str>> {
	let file = FRONTEND_FILES.get(resource.to_str().unwrap())?;
	if let Some(ext) = resource.extension() {
		if let Some(content_type) = http::ContentType::parse_flexible(ext.to_str().unwrap()) {
			return Some(content::Content(content_type, file));
		}
	}
	Some(content::Content(http::ContentType::Plain, file))
}

fn graceful_exit(err: &str) {
	eprintln!("{}", err);
	std::process::exit(1);
}
