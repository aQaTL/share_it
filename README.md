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
- By default, Share it compiles with systemd socket activation support (Linux only), and it needs 
    `libsystemd` for that. If you don't want/have it, add `--no-default-features` to the build
    command.

# libsystemd

## Ubuntu

On Ubuntu, you only need to get the systemd's dev package, like so:

```
sudo apt install libsystemd-dev
```
