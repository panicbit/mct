#![feature(plugin)]
#![plugin(docopt_macros)]
extern crate augeas;
extern crate libc;
extern crate docopt;
extern crate rustc_serialize;
extern crate unix_socket;

mod error;
mod tool;
mod mpmc;

fn main() {
    let args: Args =
        Args::docopt()
        .options_first(true)
        .decode()
        .unwrap_or_else(|e| e.exit());

    // Build sub-command commandline
    let mut cmd_args = vec![args.arg_command.clone()];
    cmd_args.extend(args.arg_args);

    match &*args.arg_command {
        "rcon" => tool::rcon::main(cmd_args),
        "start" => tool::start::main(cmd_args),
        _ => {println!("Unknown command!"); Ok(())}
    }.unwrap_or_else(|e| {
        println!("mct: {}", e);
    })
}

docopt!(Args, "
Minecraft server configuration tool

Usage:
    mct <command> [<args>...]
    mct -h

Options:
    -h, --help    Show this help

Commands:
    rcon          Configure rcon settings
    start         Start the server

See `mct <command> -h` for command specific help
");