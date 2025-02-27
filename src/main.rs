use clap::Parser;

mod register;
mod server;
mod discord;
mod env;
mod storage;
mod dsek;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    register: bool,
}

fn main() {
    let args = Args::parse();

    if args.register {
        register::start();
    } else {
        server::start().ok();
    }
}
