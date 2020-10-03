use log::{error, info};
use std::process;
use structopt::StructOpt;

mod util;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "ppa",
    about = "Command line utility to store and retrieve passwords"
)]
struct Options {
    #[structopt(short, long, help = "Enable debug logging")]
    debug: bool,

    #[structopt(subcommand)]
    command: Option<Subcommand>,
}

#[derive(Debug, StructOpt)]
enum Subcommand {
    #[structopt(about = "Initialize the store")]
    Init {},
    #[structopt(about = "Add an entry")]
    Add {
        #[structopt(short, long, help = "Name of site/service")]
        name: String,
        #[structopt(short, long, help = "Username")]
        username: String,
        #[structopt(short, long, help = "Comments")]
        comments: Option<String>,
    },
    #[structopt(about = "Search through stored entries")]
    Search {
        #[structopt(help = "Term to search for")]
        term: String,
    },
    #[structopt(about = "Remove an entry")]
    Remove {
        #[structopt(help = "Name of site/service")]
        name: String,
    },
}

fn setup_logging(debug: bool) {
    use fern::colors::{Color, ColoredLevelConfig};
    use log::LevelFilter;

    let level = if debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    let colors = ColoredLevelConfig::new().info(Color::Green);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!("{} {}", colors.color(record.level()), message))
        })
        .level(level)
        .chain(std::io::stdout())
        .apply()
        .expect("[FATAL] Could not set up logger");
}

fn main() {
    let args = Options::from_args();
    setup_logging(args.debug);

    match args.command {
        Some(Subcommand::Init {}) => match util::create_new() {
            Ok(false) => info!("Store already exists"),
            Ok(true) => info!("Store created"),
            Err(e) => {
                error!("Could not create store: {}", e);
                process::exit(1);
            }
        },
        _ => {}
    }

    let config = util::load_store();

    match args.command {
        None | Some(Subcommand::Search { .. }) => {}
        Some(Subcommand::Add { .. }) => {}
        Some(Subcommand::Remove { .. }) => {}
        _ => {}
    }
}
