use std::path::{PathBuf, Path};
use std::fs;
use rocket::*;
use rocket::{
	outcome::IntoOutcome,
	response::NamedFile,
	handler::Outcome,
	http::{Method, Status, uri::Segments},
};
use serde::Serialize;

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

impl Into<Vec<Route>> for StaticFilesBrowser {
	fn into(self) -> Vec<Route> {
		vec![Route::ranked(-3, Method::Get, "/s/<resource..>", self.clone()),
			 Route::ranked(-3, Method::Get, "/s", self)]
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

impl Handler for StaticFilesBrowser {
	fn handle<'r>(&self, req: &'r Request, _: Data) -> Outcome<'r> {
		let route = req.route().expect("route() is None");
		let is_segments_route = route.uri.path().ends_with(">");
		let path = if !is_segments_route {
			self.root.clone()
		} else {
			req.get_segments::<Segments>(1)
				.and_then(|res| res.ok())
				.and_then(|segments| segments.into_path_buf(true).ok())
				.map(|path| self.root.join(path))
				.into_outcome(Status::NotFound)?
		};

		if path.is_dir() {
			let dir_iter = fs::read_dir(path.as_path()).unwrap();
			let mut resource_dir = Vec::new();
			for entry in dir_iter.filter_map(|e| e.ok()) {
				resource_dir.push(Entry {
					e_type: if entry.path().is_dir() { Type::Dir } else { Type::File },
					name: entry.file_name().into_string().unwrap(),
				});
			}
			Outcome::from(req, response::content::Json(serde_json::to_string(&resource_dir)))
		} else {
			Outcome::from(req, NamedFile::open(&path).ok())
		}
	}
}
