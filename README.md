# Share it

Simple webapp that displays files and folders inside a given directory.

## Uploading files

You can use the `Browse` button to select files and click `Upload` to upload them into the
root directory. 

Alternatively, there's also drag-and-drop support. Simply drag and drop files into the
browser's window and they'll get uploaded right away.

# Building

- To build, you need a [Rust toolchain](https://rustup.rs/). 
    - On Linux, you might want to install `libsystemd`.
- Then, clone the repo, 
- `cd` into it 
- run the build command:

```
cargo build --release
```

- The binary will be placed at `./target/release/share_it`.

# libsystemd



# Systemd service

- Create a service unit file in `/etc/systemd/system/shareit.service` with contents:

```
[Unit]
Description=ShareIt

[Service]
ExecStart=/path/to/shareit/binary /path/to/a/dir/you/want/to/share
User=your_username

[Install]
WantedBy=mutli-user.target
```

- Create a socket file in `/etc/systemd/system/shareit.socket` with contents:

```
[Socket]
ListenStream=80
BindIPv6Only=both
Accept=no

[Install]
WantedBy=sockets.target
```

- Run `sudo systemctl daemon-reload`
- Run `sudo systemctl start shareit.socket`
- Run `sudo systemctl start shareit`
- Check if the service is running with `sudo systemctl status shareit`
