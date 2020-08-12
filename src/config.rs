use structopt::StructOpt;
use std::path::PathBuf;
use std::net::IpAddr;

pub static WELCOME_MESSAGE: &str = r#" ██████╗  ██████╗ ██╗   ██╗███████╗██████╗ ██╗   ██╗
██╔════╝ ██╔═══██╗██║   ██║██╔════╝██╔══██╗╚██╗ ██╔╝
██║  ███╗██║   ██║██║   ██║█████╗  ██████╔╝ ╚████╔╝
██║   ██║██║▄▄ ██║██║   ██║██╔══╝  ██╔══██╗  ╚██╔╝
╚██████╔╝╚██████╔╝╚██████╔╝███████╗██║  ██║   ██║
 ╚═════╝  ╚══▀▀═╝  ╚═════╝ ╚══════╝╚═╝  ╚═╝   ╚═╝"#;

#[derive(StructOpt, Debug)]
#[structopt(name = "config")]
pub struct Config {
    #[structopt(short, long, default_value="6985", env = "PORT")]
    pub port: u16,

    #[structopt(short, long, default_value="127.0.0.1", parse(try_from_str))]
    pub host: IpAddr,

    #[structopt(short, long, default_value="./data.bin", parse(from_os_str))]
    pub data: PathBuf,

    #[structopt(short, long, name="interval between saved snapshots in seconds", default_value="3600")]
    pub save_interval: u64,
}

pub fn get_conf() -> Config {
    let mut conf = None;
    let get_conf = || {
        if conf.is_none() {
            conf = Some(Config::from_args())
        }
        conf.unwrap()
    };
    get_conf()
}