// Unlinking the socket needs to also be done on startup until
// https://github.com/rust-lang/rust/issues/11203 is resolved
// i.e. until signals work

use ServerTool;
use std::io::net::pipe::{UnixListener, UnixStream};
use std::io::{Listener,Acceptor,BufferedStream};
use std::io::FileType::NamedPipe;
use std::io::fs::{stat,unlink,PathExtensions};
use std::io::process::{Command,Process};
use error::{error,wrap_error,Result};

pub struct StartTool;

impl ServerTool for StartTool {
    fn name(&self) -> &'static str {"start"}
    fn desc(&self) -> &'static str {"start a server"}
    fn main(&self, server_root: &str, args: Vec<String>) {main(server_root, args)}
}

pub fn main(server_root: &str, args: Vec<String>) {
    let _ = args;
    let server_path = &Path::new(server_root);
    let socket_path = &server_path.join("mct.sock");

    if let Err(e) = detect_running_server(socket_path) {
        println!("mct: {}", e);
        return
    }

    let mut acceptor = UnixListener::bind(socket_path).listen();

    if let Err(e) = acceptor {
        println!("mct: {}", e);
        return
    }

    if let Err(e) = spawn_server(server_path) {
        println!("mct: {}", e);
        return
    }

    for mut stream in acceptor.incoming() {
        println!("mct: incoming connection")
        match stream {
            Err(err) => {
                println!("mct: {}", err);
            }
            Ok(stream) => spawn(move || {
                handle_client(stream)
            })
        }
    }
}

fn handle_client(stream: UnixStream) {
    let mut stream = BufferedStream::new(stream);
    loop {
        match stream.read_line() {
            Ok(cmd) => print!("executing: {}", cmd),
            Err(err) => {
                println!("mct: {}", err);
                break
            }
        }
    }
}

fn detect_running_server(socket_path: &Path) -> Result<()> {
    let maybe_info = stat(socket_path);

    // try to connect to a possibly running server
    if socket_path.exists() {
        if UnixStream::connect(socket_path).is_ok() {
            Err(error("server already running"))
        } else {
            try!(unlink(socket_path));
            Ok(())
        }
    } else {
        Ok(())
    }
}

fn spawn_server(server_path: &Path) -> Result<Process> {
    let start_script = &server_path.join("StartServer.sh");

    Command::new(start_script)
        .cwd(server_path)
        .spawn()
        .map_err(|e| error(format!("{} {}", e, start_script.display().to_string()).as_slice()))
}