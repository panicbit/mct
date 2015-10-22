// Unlinking the socket needs to also be done on startup until
// https://github.com/rust-lang/rust/issues/11203 is resolved
// i.e. until signals work

use std::path::Path;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::sync::{Arc, Mutex};
use unix_socket::{UnixListener, UnixStream};
use std::fs::{self, remove_file};
use std::process::{Command,Child};
use std::thread::spawn;
use docopt;
use common::{error,Result};
use mpmc::MultiSender;

pub fn main(args: Vec<String>) -> Result<()> {
    // Parse arguments
    let args: Args =
        Args::docopt()
        .argv(args.into_iter())
        .decode()
        .unwrap_or_else(|e| e.exit());

    let server_root = &Path::new(&args.arg_server_root);

    try!(start(server_root));

    Ok(())
}

fn start<P: AsRef<Path>>(server_root: P) -> Result<()> {
    let server_root = server_root.as_ref();
    let socket_path = &server_root.join("mct.sock");

    // The server may not already be running
    try!(detect_running_server(socket_path));

    // Create a socket and spawn the server
    let socket = try!(UnixListener::bind(socket_path));
    let server = try!(spawn_server(server_root));

    // Acquire the server's stdout/stdin
    let server_out = try!(server.stdout.ok_or(error("Server stdout unavailable")));
    let server_in = try!(server.stdin.ok_or(error("Server stdin unavailable")));

    // Buffer them
    let mut server_out = BufReader::new(server_out);
    let server_in = BufWriter::new(server_in);

    // Make server input shareable
    let server_in = Arc::new(Mutex::new(server_in));

    let mut stdout_sender = MultiSender::new();

    // Read the stdout of the server and broadcast it
    let main_thread = {
        let stdout_sender = stdout_sender.clone();
        spawn(move || {
            let mut stdout_sender = stdout_sender.clone();
            let ref mut line = String::new();
            while server_out.read_line(line).is_ok() && !line.is_empty() {
                stdout_sender.send(line.clone());
                line.clear();
            }
            stdout_sender.disconnect_all();
        })
    };

    // Accept clients
    // This thread may not be joined, otherwise the prorgram won't exit
    // after the server has shut down
    spawn(move || {
        while let Ok(client) = socket.accept() {
            let stdout_receiver = stdout_sender.subscribe();
            let mut client_in = client.try_clone().unwrap();
            let mut client_out = BufReader::new(client);

            // Sends server stdout to client
            spawn(move || -> io::Result<()> {
                for line in stdout_receiver {
                    try!(client_in.write_all(line.as_bytes()));
                    try!(client_in.flush());
                }
                Ok(())
            });

            // Sends client stdin to server
            let server_in = server_in.clone();
            spawn(move || -> io::Result<()> {
                let ref mut line = String::new();
                while client_out.read_line(line).is_ok() && !line.is_empty() {
                    let mut server_in = server_in.lock().unwrap();
                    try!(server_in.write_all(line.as_bytes()));
                    try!(server_in.flush());
                    line.clear();
                }
                Ok(())
            });

        }
    });

    main_thread.join().unwrap();

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
    -q, --quiet   Don't print server messages
");