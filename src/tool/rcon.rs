use docopt;
use ::augeas;
use augeas::{Augeas,AugFlag};
use std::ffi::NulError;
use std::process::exit;
use common;

/// Stores rcon properties
#[derive(Debug)]
pub struct RconInfo {
    pub enabled: bool,
    pub port: u16,
    pub pass: String
}

impl RconInfo {
    /// Reads rcon properties using augeas
    pub fn from_augeas(aug: &Augeas) -> augeas::Result<RconInfo> {
        let rcon_enabled = try!(aug.get("server.properties/enable-rcon"))
            .and_then(|enabled| enabled.parse::<bool>().ok())
            .unwrap_or(false);
        let rcon_port = try!(aug.get("server.properties/rcon.port"))
            .and_then(|port| port.parse::<u16>().ok())
            .unwrap_or(25575);
        let rcon_pass = try!(aug.get("server.properties/rcon.password"))
            .unwrap_or("".to_string());

        Ok(RconInfo {
            enabled: rcon_enabled,
            port: rcon_port,
            pass: rcon_pass
        })
    }

    /// Updates rcon properties in augeas BUT does not save them
    pub fn update_augeas(&self, aug: &mut Augeas) -> augeas::Result<()> {
        try!(aug.set("server.properties/enable-rcon", &self.enabled.to_string()));
        try!(aug.set("server.properties/rcon.port", &self.port.to_string()));
        try!(aug.set("server.properties/rcon.password", &self.pass.to_string()));
        Ok(())
    }
}

pub fn main(args: Vec<String>) -> common::Result<()> {
    // Parse arguments
    let args: Args =
        Args::docopt()
        .argv(args.into_iter())
        .decode()
        .unwrap_or_else(|e| e.exit());
    let server_root = &args.arg_server_root;

    // Read rcon properties
    let aug = &mut Augeas::new(server_root, "res/augeas/", AugFlag::None).unwrap();
    let mut rcon = RconInfo::from_augeas(aug).unwrap();

    // cmd: rcon show
    if args.cmd_show {
        cmd_show(rcon);
    }

    // cmd: rcon edit
    if args.cmd_edit {
        // Update rcon port
        if args.arg_port {
            let new_port = args.arg_port.parse::<u16>();
            rcon.port = new_port.unwrap_or_else(|_| {
                println!("invalid port");
                exit(0);
            });
        }



        // TODO: implement remaining flags

        rcon.update_augeas(aug).unwrap();
        aug.save();
    }

    Ok(())
}

/// Prints all rcon properties
fn cmd_show(rcon: RconInfo) {
    println!(
r"enabled: {}
port: {}
pass: {}",
    rcon.enabled,
    rcon.port,
    rcon.pass)
}

docopt!(Args derive Debug, "
Configure rcon settings

Usage:
    rcon <server-root> enable
    rcon <server-root> disable
    rcon <server-root> show
    rcon <server-root> edit [options]

Options:
    -h, --help           Show this help
    -g, --genpass        Generate a new password
    -p, --port           Set a new port
    --pw=<pw>            Set a new password
");