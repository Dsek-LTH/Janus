use clap::Parser;
// use tokio::task::futures;

mod register;
mod server;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    register: bool,
}

fn main() {
    let args = Args::parse();

    if args.register {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(register::register());        
    } else {
        // server::start();
    }
}