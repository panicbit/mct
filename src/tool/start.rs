// Unlinking the socket needs to also be done on startup until
// https://github.com/rust-lang/rust/issues/11203 is resolved
// i.e. until signals work

use ServerTool;
use std::io::net::pipe::{UnixListener, UnixStream};
use std::io::{Listener,Acceptor,BufferedStream};
use std::io::net::pipe::UnixAcceptor;
use std::io::FileType::NamedPipe;
use std::io::fs::{unlink,PathExtensions};
use std::io::process::{Command,Process};
use std::io::pipe::PipeStream;
use std::collections::HashMap;
use std::slice::SliceExt;
use std::mem::drop;
use error::{error,Result};
use broadcast::BroadcastStation;

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

    let mut acceptor = match UnixListener::bind(socket_path).listen() {
        Ok(acceptor) => acceptor,
        Err(e) => {
            println!("mct: {}", e);
            return
        }
    };

    let mut server = match spawn_server(server_path) {
        Ok(server) => server,
        Err(e) => {
            println!("mct spawn server: {}", e);
            return
        }
    };

    let mut server_stdout = BufferedStream::new(server.stdout.clone().unwrap());
    let server_stdin = BufferedStream::new(server.stdin.clone().unwrap());

    let (cmd_tx, cmd_rx) = channel();
    let mut station = BroadcastStation::<String>::new();

    {
        let station1 = station.clone();
        let station2 = station.clone();
        let acceptor = acceptor.clone();
        spawn(move || server_console_broadcaster(server_stdout, station1, acceptor));
        spawn(move || server_cmd_executor(server_stdin, cmd_rx, station2));
    }

    let connected_clients = HashMap::<uint, UnixStream>::new();

    for mut stream in acceptor.incoming() {
        match stream {
            Err(err) => {
                //println!("mct: {}", err)
                println!("mct stopping")
                server.signal_exit();
                break
            }
            Ok(stream) => {
                let cmd_tx = cmd_tx.clone();
                let stream1 = stream.clone();
                let stream2 = stream.clone();
                let mut console_rx = station.signup();
                spawn(move || client_cmd_receiver(stream1, cmd_tx));
                spawn(move || client_console_sender(stream2, console_rx))
            }
        }
    }
}

fn client_cmd_receiver(stream: UnixStream, cmd_tx: Sender<String>) {
    let mut stream = BufferedStream::new(stream);
    loop {
        match stream.read_line() {
            Ok(cmd) => cmd_tx.send(cmd),
            Err(err) => {
                println!("mct: {}", err);
                break
            }
        }
    }
}

fn client_console_sender(mut stream: UnixStream, console_rx: Receiver<String>) {
    loop {
        match console_rx.recv_opt() {
            Ok(output) => stream.write_str(output.as_slice()),
            Err(_) => break
        };
    }
}

fn server_cmd_executor(mut server_stdin: BufferedStream<PipeStream>, cmd_rx: Receiver<String>, mut console_station: BroadcastStation<String>) {
    loop {
        match cmd_rx.recv_opt() {
            Ok(cmd) => {
                let msg = format!("mct: executing '{}'\n", cmd.as_slice().trim_right_chars('\n'));
                print!("{}", msg.as_slice());
                console_station.send(msg);
                server_stdin.write_str(cmd.as_slice());
                server_stdin.flush();
            },
            Err(()) => {
                println!("mct: stopping cmd executor");
                break
            } 
        }
    }
}

fn server_console_broadcaster(mut server_stdout: BufferedStream<PipeStream>, mut station: BroadcastStation<String>, mut acceptor: UnixAcceptor) {
    loop {
        match server_stdout.read_line() {
            Ok(line) => {
                print!("{}", line);
                station.send(line);
            },
            Err(_) => {
                acceptor.close_accept();
                println!("mct: stopping console broadcaster")
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