use ServerTool;
use augeas::{Augeas,AugFlags};

pub struct StartTool;

impl ServerTool for StartTool {
    fn name(&self) -> &'static str {"start"}
    fn desc(&self) -> &'static str {"start a server"}
    fn main(&self, server_root: &str, args: Vec<String>) {main(server_root, args)}
}

pub fn main(server_root: &str, mut args: Vec<String>) {
    let aug = &mut Augeas::new(server_root, "res/augeas/", AugFlags::None);
    let maybe_cmd = args.remove(1);
    let maybe_cmd_slice = maybe_cmd.as_ref().map(|s| s.as_slice());

}