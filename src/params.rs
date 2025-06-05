use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// SSH port of the remote host
    #[arg(short = 'p', long = "ssh-port", default_value_t = 10022)]
    pub ssh_port: u32,

    /// SSH destination host (IP or hostname)
    #[arg(short = 'd', long = "ssh-dest", default_value_t = String::from("127.0.0.1"))]
    pub ssh_dest: String,

    /// Use sudo/root privileges on remote host
    #[arg(long, default_value_t = true)]
    pub use_sudo: bool,
}
