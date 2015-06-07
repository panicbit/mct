// Unlinking the socket needs to also be done on startup until
// https://github.com/rust-lang/rust/issues/11203 is resolved
// i.e. until signals work

use std::path::Path;
use std::io::{Stdin, Stdout, BufStream, BufReader, BufWriter};
use std::sync::mpsc::{channel,Sender,Receiver};
use unix_socket::{self, UnixListener, UnixStream};
use std::fs::{remove_file,PathExt};
use std::process::{Command,Child};
use std::thread::Thread;
use docopt;
use error::{error,Result};
use mpmc::MultiSender;

pub fn main(args: Vec<String>) {
    let args: Args =
        Args::docopt()
        .argv(args.into_iter())
        .decode()
        .unwrap_or_else(|e| e.exit());
    let server_root = &args.arg_server_root;
    let server_path = &Path::new(server_root);
    let socket_path = &server_path.join("mct.sock");

    if let Err(e) = detect_running_server(socket_path) {
        println!("mct: {:?}", e);
        return
    }

    let mut listener = match UnixListener::bind(socket_path) {
        Ok(listener) => listener,
        Err(e) => {
            println!("mct: {:?}", e);
            return
        }
    };

    let mc_server = match spawn_server(server_path) {
        Ok(server) => server,
        Err(e) => {
            println!("mct spawn server: {:?}", e);
            return
        }
    };

    let server_stdout = BufReader::new(mc_server.stdout.clone().unwrap());
    let server_stdin = BufWriter::new(mc_server.stdin.clone().unwrap());

    let (cmd_tx, cmd_rx) = channel();
    let mut station = MultiSender::<String>::new();

    {
        let station1 = station.clone();
        let station2 = station.clone();
        let listener = listener.clone();
        Thread::spawn(move || server_console_broadcaster(server_stdout, station1, listener));
        Thread::spawn(move || server_cmd_executor(server_stdin, cmd_rx, station2));
    }

    for mut stream in listener.incoming() {
        match stream {
            Err(_) => {
                //println!("mct: {}", err)
                println!("mct stopping");
                println!("disconnecting clients");
                break
            }
            Ok(stream) => {
                let cmd_tx = cmd_tx.clone();
                let stream1 = stream.clone();
                let stream2 = stream.clone();
                let console_rx = station.receiver();
                Thread::spawn(move || client_cmd_receiver(stream1, cmd_tx));
                Thread::spawn(move || client_console_sender(stream2, console_rx));
            }
        }
    }
}

fn client_cmd_receiver(stream: UnixStream, cmd_tx: Sender<String>) {
    let mut stream = BufStream::new(stream);
    loop {
        match stream.read_line() {
            Ok(cmd) => if let Err(err) = cmd_tx.send(cmd) {
                println!("mct: {:?}", err);
                break
            },
            Err(err) => {
                println!("mct: {:?}", err);
                break
            }
        }
    }
}

fn client_console_sender(mut stream: UnixStream, console_rx: Receiver<String>) {
    loop {
        match console_rx.recv() {
            Ok(output) => {let _ = stream.write_str(output.as_slice());},
            Err(_) => break
        };
    }
}

fn server_cmd_executor(mut server_stdin: BufWriter<Stdout>, cmd_rx: Receiver<String>, mut console_station: MultiSender<String>) {
    loop {
        match cmd_rx.recv() {
            Ok(cmd) => {
                let msg = format!("mct: executing '{}'\n", cmd.as_slice().trim_right_matches('\n'));
                print!("{}", msg.as_slice());
                console_station.send(msg);
                let _ = server_stdin.write_str(cmd.as_slice());
                let _ = server_stdin.flush();
            },
            Err(_) => {
                println!("mct: stopping cmd executor");
                break
            } 
        }
    }
}

fn server_console_broadcaster(mut server_stdout: BufReader<Stdin>, mut station: MultiSender<String>, mut listener: unix_socket::Incoming) {
    loop {
        match server_stdout.read_line() {
            Ok(line) => {
                print!("{}", line);
                station.send(line);
            },
            Err(_) => {
                let _ = listener.close_accept();
                println!("mct: stopping console broadcaster");
                break
            }
        }
    }
}

fn detect_running_server(socket_path: &Path) -> Result<()> {
    // try to connect to a possibly running server
    if socket_path.exists() {
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
        .cwd(server_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| error(format!("{} {}", e, start_script.display().to_string()).as_slice()))
}

docopt!(Args, "
Start a server

Usage:
    start <server-root> [options]

Options:
    -h, --help    Show this help
");