use actix_service::{Service, ServiceFactory};
use actix_web::dev::{
	AppService, HttpServiceFactory, ResourceDef, ServiceRequest, ServiceResponse,
};
use actix_web::http::{header, Method, StatusCode};
use actix_web::{HttpResponse, Responder, ResponseError};
use futures::future::{ok, ready, LocalBoxFuture};
use log::info;
use serde::Serialize;
use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};
use std::task::{Context, Poll};

#[derive(Clone)]
pub struct StaticFilesBrowser {
	root: PathBuf,
}

impl StaticFilesBrowser {
	pub fn new(path: impl AsRef<Path>) -> Self {
		StaticFilesBrowser {
			root: path.as_ref().into(),
		}
	}
}

#[derive(Serialize)]
struct Entry {
	e_type: Type,
	name: String,
}

#[derive(Serialize)]
enum Type {
	File,
	Dir,
}

#[derive(Debug)]
enum StaticFilesBrowserError {
	NotFound,
}

impl Display for StaticFilesBrowserError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl ResponseError for StaticFilesBrowserError {
	fn status_code(&self) -> StatusCode {
		use StaticFilesBrowserError::*;
		match self {
			NotFound => StatusCode::NOT_FOUND,
		}
	}

	fn error_response(&self) -> HttpResponse {
		use StaticFilesBrowserError::*;
		match self {
			NotFound => crate::not_found(),
		}
	}
}

impl StaticFilesBrowser {
	fn handle(&self, req: ServiceRequest) -> Result<ServiceResponse, actix_web::Error> {
		let (http_req, _) = req.into_parts();

		let path = {
			let path = percent_encoding::percent_decode_str(http_req.uri().path()).decode_utf8()?;
			let path = path
				.strip_prefix("/s")
				.ok_or(StaticFilesBrowserError::NotFound)?;
			let path = path.strip_prefix('/').unwrap_or(path);
			self.root.join(path)
		};

		if !path
			.canonicalize()
			.map(|abs_path| abs_path.starts_with(&self.root))
			.unwrap_or_default()
		{
			return Err(StaticFilesBrowserError::NotFound.into());
		}

		info!("serving {:?}", path);

		if path.is_dir() {
			let dir_iter = fs::read_dir(path.as_path()).unwrap();
			let mut resource_dir = Vec::new();
			for entry in dir_iter.filter_map(|e| e.ok()) {
				resource_dir.push(Entry {
					e_type: if entry.path().is_dir() {
						Type::Dir
					} else {
						Type::File
					},
					name: entry.file_name().into_string().unwrap(),
				});
			}

			let response = HttpResponse::Ok().json(resource_dir).respond_to(&http_req);
			Ok(ServiceResponse::new(http_req, response))
		} else {
			let response = actix_files::NamedFile::open(path)?.into_response(&http_req);
			Ok(ServiceResponse::new(http_req, response))
		}
	}
}

impl HttpServiceFactory for StaticFilesBrowser {
	fn register(self, config: &mut AppService) {
		config.register_service(
			ResourceDef::new(["/s", "/s/", "/s/{resource..}"]),
			Some(vec![Box::new(actix_web::guard::Get())]),
			self,
			None,
		)
	}
}

impl ServiceFactory<ServiceRequest> for StaticFilesBrowser {
	type Response = ServiceResponse;
	type Error = actix_web::Error;
	type Config = ();
	type Service = Self;
	type InitError = ();
	type Future = LocalBoxFuture<'static, Result<Self::Service, Self::InitError>>;

	fn new_service(&self, _: Self::Config) -> Self::Future {
		Box::pin(ready(Ok(self.clone())))
	}
}

impl Service<ServiceRequest> for StaticFilesBrowser {
	type Response = ServiceResponse;
	type Error = actix_web::Error;
	type Future = LocalBoxFuture<'static, Result<ServiceResponse, actix_web::Error>>;

	fn poll_ready(&self, _ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		Poll::Ready(Ok(()))
	}

	fn call(&self, req: ServiceRequest) -> Self::Future {
		if !matches!(*req.method(), Method::GET) {
			return Box::pin(ok(req.into_response(
				actix_web::HttpResponse::MethodNotAllowed()
					.insert_header(header::ContentType::plaintext())
					.body("Request did not meet this resource's requirements."),
			)));
		}
		Box::pin(ready(self.handle(req)))
	}
}
