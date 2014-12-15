extern crate getopts;
extern crate augeas;
//use getopts::{optflag, usage};
use std::os;

mod tool {
    pub mod rcon;
}

pub trait ServerTool {
    fn name(&self) -> &'static str;
    fn desc(&self) -> &'static str;
    fn main(&self, server_root: &str, args: Vec<String>);
}

fn main() {
    let tools = &[
        tool::rcon::RconTool
    ];

    let mut args = os::args();

    let maybe_root = args.remove(1);

    let maybe_tool = args.remove(1).and_then(|tool_name| {
        tools
            .iter()
            .skip_while(|tool| tool.name() != tool_name)
            .next()
    });

    match (maybe_root, maybe_tool) {
        (Some(root),Some(tool)) => tool.main(root.as_slice(), args),
        _ => show_help(tools)
    }
}

fn show_help<T: ServerTool>(tools: &[T]) {
    println!("Available tools:");
    for tool in tools.iter() {
        println!("\t{}\t{}", tool.name(), tool.desc());
    }
}