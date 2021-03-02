use std::collections::HashMap;

pub type FrontendFiles = HashMap<&'static str, &'static str>;

lazy_static::lazy_static! {
	pub static ref FRONTEND_FILES: FrontendFiles = include!("generated/frontend_files.array").iter().cloned().collect();
}
