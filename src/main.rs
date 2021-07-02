#![feature(proc_macro_hygiene, decl_macro, array_methods)]

use anyhow::{bail, Context, Result};
use rocket::{
	config::{Config, Environment, Limits, LoggingLevel},
	get, http, post,
	response::content,
	Data, State,
};
use std::{fs, path::PathBuf};

mod frontend;
mod staticfiles;

use crate::{frontend::*, staticfiles::*};

fn clap_app() -> clap::App<'static, 'static> {
	use clap::*;
	App::new(crate_name!())
		.version(crate_version!())
		.about(crate_description!())
		.arg(
			Arg::with_name("resource")
				.required(true)
				.multiple(true)
				.takes_value(true)
				.index(1)
				.default_value("."),
		)
		.arg(
			Arg::with_name("name")
				.help("e.g. `--name foo` will result in sharing the resource on `/s/foo`")
				.short("n")
				.takes_value(true)
				.empty_values(true)
				.default_value(""),
		)
		.arg(
			Arg::with_name("address")
				.help("ip of the network interface on which the application will serve")
				.long("address")
				.takes_value(true)
				.default_value("127.0.0.1"),
		)
		.arg(
			Arg::with_name("port")
				.help("port on which the application will listen")
				.short("p")
				.long("port")
				.takes_value(true)
				.default_value("80"),
		)
}

#[cfg(all(target_os = "linux", feature = "systemd"))]
mod systemd {
	use anyhow::{bail, Result};
	use std::net::TcpListener;
	use std::os::unix::prelude::FromRawFd;

	const SD_LISTEN_FDS_START: i32 = 3;

	#[link(name = "systemd")]
	extern "C" {
		fn sd_listen_fds(unset_environment: i32) -> i32;
	}

	pub fn systemd_socket_activation() -> Result<Option<Vec<TcpListener>>> {
		let nfds = unsafe { sd_listen_fds(false as i32) };
		if nfds < 0 {
			bail!(
				"sd_listen_fds failed: {:?}",
				std::io::Error::from_raw_os_error(nfds)
			);
		}
		if nfds == 0 {
			return Ok(None);
		}

		let listeners: Vec<TcpListener> = (SD_LISTEN_FDS_START..(SD_LISTEN_FDS_START + nfds))
			.map(|fd| unsafe { TcpListener::from_raw_fd(fd) })
			.collect();

		Ok(Some(listeners))
	}
}

fn main() -> Result<()> {
	let app = clap_app().get_matches();
	let port = app.value_of("port").unwrap();
	let port = port.parse::<u16>().with_context(|| {
		format!(
			"Invalid port value, could not parse `{}` as a port number (0 - 65535)",
			port
		)
	})?;

	let address = app.value_of("address").unwrap();
	let name = app.value_of("name").unwrap();

	let resource_str = app.value_of("resource").unwrap();
	let resource = PathBuf::from(resource_str);
	if !resource.exists() {
		bail!("{} not found", resource_str);
	}
	let mut resource = resource.canonicalize().unwrap();
	if resource.is_file() {
		resource.pop();
	}

	let resource_dir: Vec<_> = fs::read_dir(resource.clone())
		.with_context(|| format!("error reading {:?}", resource,))?
		.collect();

	println!(
		"Sharing {} on {}:{}/{}/",
		resource.file_name().unwrap().to_string_lossy(),
		address,
		port,
		name
	);

	let config = Config::build(Environment::Production)
		.address(address)
		.port(port)
		.log_level(LoggingLevel::Normal)
		// 1GiB file upload limit
		.limits(Limits::new().limit("forms", 1 * 1024 * 1024 * 1024))
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

	let mut routes = rocket::routes![index, serve_frontend, upload];
	routes.append(&mut StaticFilesBrowser::new(resource.clone()).into());

	Result::<(), _>::Err(
		rocket::custom(config)
			.manage(ResourceDir(resource))
			.mount("/", routes)
			.launch(),
	)?;

	Ok(())
}

struct ResourceDir(PathBuf);

#[get("/")]
fn index() -> content::Html<&'static [u8]> {
	content::Html(FRONTEND_FILES.get("index.html").unwrap())
}

#[get("/<resource..>", rank = 2)]
fn serve_frontend(resource: PathBuf) -> Option<content::Content<&'static [u8]>> {
	let file = FRONTEND_FILES.get(resource.to_str().unwrap())?;
	if let Some(ext) = resource.extension() {
		if let Some(content_type) = http::ContentType::parse_flexible(ext.to_str().unwrap()) {
			return Some(content::Content(content_type, file));
		}
	}
	Some(content::Content(http::ContentType::Plain, file))
}

#[post("/upload/<filename..>", data = "<file>")]
fn upload(
	filename: PathBuf,
	file: Data,
	resource_dir: State<ResourceDir>,
) -> Result<(), std::io::Error> {
	let file_path = resource_dir.0.join(filename);
	file.stream_to_file(file_path)?;

	Ok(())
}
