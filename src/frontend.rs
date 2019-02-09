use std::collections::HashMap;

pub type FrontendFiles = HashMap<&'static str, &'static str>;

pub fn frontend_files() -> FrontendFiles {
	let array = include!("generated/frontend_files.array");
	array.iter().cloned().collect()
}
