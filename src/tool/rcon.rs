use ServerTool;
use augeas::{Augeas,AugFlags};

pub struct RconTool;

impl ServerTool for RconTool {
    fn name(&self) -> &'static str {"rcon"}
    fn desc(&self) -> &'static str {"configure rcon related things"}
    fn main(&self, server_root: &str, args: Vec<String>) {main(server_root, args)}
}

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

pub fn main(server_root: &str, mut args: Vec<String>) {
    let aug = &mut Augeas::new(server_root, "res/augeas/", AugFlags::None);
    let rcon = RconInfo::from_augeas(aug);
    let maybe_cmd = args.remove(1);
    let maybe_cmd_slice = maybe_cmd.as_ref().map(|s| s.as_slice());

    match maybe_cmd_slice {
        Some("show") => cmd_show(rcon),
        _ => show_help(args)
    };
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

fn show_help(mut args: Vec<String>) {
    println!(
r"Usage: {} <server_root> rcon <command>

Available commands:
    show    print rcon config
",
    args.remove(0).unwrap())
}