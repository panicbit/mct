// Unlinking the socket needs to also be done on startup until
// https://github.com/rust-lang/rust/issues/11203 is resolved
// i.e. until signals work

use std::path::Path;
use std::io::{Stdin, Stdout, BufStream, BufReader, BufWriter};
use std::sync::mpsc::{channel,Sender,Receiver};
use unix_socket::{self, UnixListener, UnixStream};
use std::fs::{self, remove_file};
use std::process::{Command,Child};
use std::thread::Thread;
use docopt;
use error::{error,Result};
use mpmc::MultiSender;

pub fn main(args: Vec<String>) -> Result<()> {
    let args: Args =
        Args::docopt()
        .argv(args.into_iter())
        .decode()
        .unwrap_or_else(|e| e.exit());
    let server_root = &args.arg_server_root;
    let server_path = &Path::new(server_root);
    let socket_path = &server_path.join("mct.sock");

    try!(detect_running_server(socket_path));

    let mut server = try!(spawn_server(server_path));

    server.wait();

    Ok(())
}

fn detect_running_server(socket_path: &Path) -> Result<()> {
    // try to connect to a possibly running server
    let socket_exists = fs::metadata(socket_path).is_ok();
    if socket_exists {
        if UnixStream::connect(socket_path).is_ok() {
            Err(error("server already running"))
        } else {
            try!(remove_file(socket_path));
            Ok(())
        }
    } else {
        Ok(())
    }
}

fn spawn_server(server_path: &Path) -> Result<Child> {
    use std::process::Stdio;
    let start_script = &server_path.join("StartServer.sh");

    Command::new(start_script)
        .current_dir(server_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| error(format!("{}: {}", start_script.display(), e)))
}

docopt!(Args, "
Start a server

Usage:
    start <server-root> [options]

Options:
    -h, --help    Show this help
");