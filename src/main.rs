#![feature(proc_macro_hygiene, decl_macro, array_methods)]

use actix_web::{
	get,
	http::header,
	middleware::Logger,
	post,
	web::{self, PayloadConfig},
	App, HttpResponse, HttpServer, Responder,
};
use anyhow::{bail, Context, Result};
use futures::StreamExt;
use log::info;
use std::{fs, path::PathBuf};
use tokio::io::AsyncWriteExt;

mod frontend;
mod staticfiles;
#[cfg(all(target_os = "linux", feature = "systemd"))]
mod systemd;

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

fn main() -> Result<()> {
	flexi_logger::Logger::try_with_str("info")?.start()?;

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

	info!(
		"Sharing {} on {}:{}/{}/",
		resource.file_name().unwrap().to_string_lossy(),
		address,
		port,
		name
	);

	let resource_dir = resource_dir
		.into_iter()
		.filter_map(|e| e.ok())
		.map(|e| e.file_name().into_string().unwrap())
		.collect::<Vec<_>>();

	for e in &resource_dir {
		info!("{}", e);
	}

	actix_web::rt::System::new().block_on(async move {
		let mut http_server = HttpServer::new(move || {
			App::new()
				.app_data(web::Data::new(ResourceDir(resource.clone())))
				// 1GiB file upload limit
				.app_data(PayloadConfig::new(1024 * 1024 * 1024))
				.wrap(Logger::new(r#"%a "%r" %s %b %T"#))
				.service(index)
				.service(StaticFilesBrowser::new(resource.clone()))
				.service(upload)
				.service(serve_frontend)
				.default_service(web::to(not_found))
		});

		#[cfg(all(target_os = "linux", feature = "systemd"))]
		{
			match systemd::systemd_socket_activation()? {
				Some(sockets) => {
					info!("Using systemd provided sockets instead");
					for socket in sockets {
						http_server = http_server.listen(socket)?;
					}
				}
				None => {
					http_server = http_server.bind(format!("{}:{}", address, port))?;
				}
			}
		}
		#[cfg(not(all(target_os = "linux", feature = "systemd")))]
		{
			http_server = http_server.bind(format!("{}:{}", address, port))?;
		}

		http_server.run().await?;

		Result::<()>::Ok(())
	})?;

	Ok(())
}

struct ResourceDir(PathBuf);

#[get("/")]
async fn index() -> HttpResponse {
	let index_html: &'static [u8] = *FRONTEND_FILES.get("index.html").unwrap();
	HttpResponse::Ok()
		.insert_header(header::ContentType::html())
		.body(index_html)
}

pub fn not_found() -> HttpResponse {
	HttpResponse::NotFound().body(
		r####"<html>
<head>
	<title>404 Not Found</title>
<head>
<body>
	<center>
		<h1>404 Not Found</h1>
		<hr>
		<h3>Share It</h1>
	</center>
</body>
</html>"####,
	)
}

#[get("/{resource..}")]
async fn serve_frontend(resource: web::Path<PathBuf>) -> Option<HttpResponse> {
	let resource = resource.into_inner();
	info!("Serving frontend file {:?}", resource);
	let file: &'static [u8] = *FRONTEND_FILES.get(resource.to_str().unwrap())?;
	let mut response = HttpResponse::Ok();

	let mime_guess = mime_guess::from_path(resource.as_path());
	if let Some(mime_guess) = mime_guess.first() {
		response.content_type(mime_guess);
	}

	Some(response.body(file))
}

#[post("/upload/{filename..}")]
async fn upload(
	filename: web::Path<PathBuf>,
	resource_dir: web::Data<ResourceDir>,
	payload: web::Payload,
) -> impl Responder {
	let filename = filename.into_inner();
	let mut payload = payload.into_inner();

	let file_path = resource_dir.get_ref().0.join(filename);

	let mut file: tokio::fs::File = tokio::fs::File::create(&file_path).await?;

	info!("Writing to file {:?}", file_path);

	while let Some(chunk) = payload.next().await {
		let chunk = chunk?;
		file.write_all(chunk.as_ref()).await?;
	}

	Result::<_, actix_web::Error>::Ok("")
}
