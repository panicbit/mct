#![feature(plugin)]
extern crate getopts;
extern crate augeas;
extern crate libc;
#[plugin]
extern crate docopt_macros;
extern crate docopt;
extern crate rustc_serialize;

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
    cmd_args.push_all(args.arg_args.as_slice());

    match args.arg_command.as_slice() {
        "rcon" => tool::rcon::main(cmd_args),
        "start" => tool::start::main(cmd_args),
        _ => println!("Unknown command!")
    };
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