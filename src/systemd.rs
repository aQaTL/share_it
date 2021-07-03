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
