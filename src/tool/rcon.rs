use docopt;
use augeas::{Augeas,AugFlags};

#[deriving(Show)]
pub struct RconInfo {
    pub enabled: bool,
    pub port: u16,
    pub pass: String
}

impl RconInfo {
    pub fn from_augeas(aug: &Augeas) -> RconInfo {
        let rcon_enabled = aug
            .get("server.properties/enable-rcon")
            .and_then(|enabled| from_str::<bool>(enabled.as_slice()))
            .unwrap_or(false);
        let rcon_port = aug
            .get("server.properties/rcon.port")
            .and_then(|port| from_str::<u16>(port.as_slice()))
            .unwrap_or(25575);
        let rcon_pass = aug.get("server.properties/rcon.password")
            .unwrap_or("".to_string());

        RconInfo {
            enabled: rcon_enabled,
            port: rcon_port,
            pass: rcon_pass
        }
    }

    pub fn update_augeas(&self, aug: &mut Augeas) {
        aug.set("server.properties/enable-rcon", self.enabled.to_string().as_slice());
        aug.set("server.properties/rcon.port", self.port.to_string().as_slice());
        aug.set("server.properties/rcon.password", self.pass.to_string().as_slice());
    }
}

pub fn main(mut args: Vec<String>) {
    let args: Args =
        Args::docopt()
        .argv(args.into_iter())
        .decode()
        .unwrap_or_else(|e| e.exit());
    let server_root = args.arg_server_root.as_slice();
    let aug = &mut Augeas::new(server_root, "res/augeas/", AugFlags::None);
    let rcon = RconInfo::from_augeas(aug);

    if args.flag_list {
        cmd_show(rcon);
    }
}

fn cmd_show(rcon: RconInfo) {
    println!(
r"enabled: {}
   port: {}
   pass: '{}'",
    rcon.enabled,
    rcon.port,
    rcon.pass)
}

docopt!(Args deriving Show, "
Configure rcon settings

Usage:
    rcon <server-root> [options]

Options:
    -h, --help           Show this help
    -l, --list           List settings
    -g, --genpass        Generate a new password
    -p, --port=<port>    Set a new port
    --pass=<pw>          Set a new password
");