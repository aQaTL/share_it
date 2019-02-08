#![feature(proc_macro_hygiene, decl_macro)]

use rocket::{
	request, response, get,
	config::{Config, Environment},
};
use rocket_contrib::serve::StaticFiles;
use std::{
	path::{Path, PathBuf},
	fs,
};

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
			.short("ip")
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
			exit_gracefully(Some(&format!(
				"Invalid value, could not parse `{}` as a port number (0 - 65535)", port)));
			0
		}
	};

	let address = app.value_of("address").unwrap();
	let name = app.value_of("name").unwrap();
	let mut resource = PathBuf::from(app.value_of("resource").unwrap()).canonicalize().unwrap();
	if resource.is_file() {
		resource.pop();
	}

	let resource_dir = match fs::read_dir(resource.clone()) {
		Ok(dir) => dir.collect::<Vec<_>>(),
		Err(err) => {
			exit_gracefully(Some(&format!("error reading {}: {}",
										  resource.display(), err.to_string())));
			unreachable!()
		}
	};

	println!("Sharing {} on {}:{}/{}",
			 resource.file_name().unwrap().to_string_lossy(), address, port, name);

	let config = Config::build(Environment::Production)
		.address(address)
		.port(port)
		.finalize()
		.unwrap();


	for f in resource_dir {
		println!("{:?}", f);
	}

	let mut static_files: Vec<rocket::Route> = StaticFiles::new(
		resource, rocket_contrib::serve::Options::DotFiles).into();
	static_files.push(rocket::Route::new(rocket::http::Method::Get, "/", get_dir_ls));

	rocket::custom(config)
		.mount(&format!("/{}", name), static_files)
		.mount("/", rocket::routes![index])
		.launch();
}

#[get("/")]
fn index() -> String {
	"this is index file".to_string()
}

fn get_dir_ls<'r>(req: &'r request::Request, _: rocket::Data) -> rocket::handler::Outcome<'r> {
	rocket::Outcome::from(req, "dirs...".to_string())
}


fn exit_gracefully(err: Option<&str>) {
	std::process::exit(match err {
		Some(err) => {
			eprintln!("{}", err);
			1
		}
		None => 0
	});
}
