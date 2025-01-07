use clap::Parser;

/// synchronize your walpaper and terminal colorscheme with the track image you're listening to on spotify
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// add api credentials
    #[arg(short, long, default_value_t = false)]
    add_key: bool,
}

mod credentials;
mod sync;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.add_key {
        credentials::run().await;
    } else {
        sync::run().await;
    }
}
