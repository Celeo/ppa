use clipboard::{ClipboardContext, ClipboardProvider};
use dialoguer::{theme::ColorfulTheme, Password};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use log::{debug, error, info, warn};
use prettytable::{cell, format, row, Table};
use std::process;
use structopt::StructOpt;

mod util;
use util::{CopyWhat, Entry};

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
        #[structopt(help = "Term to search for; leave blank to list all")]
        term: Option<String>,
    },
    #[structopt(about = "Copy a username or password to your clipboard")]
    Copy {
        #[structopt(help = "Name of site/service")]
        name: String,
        #[structopt(possible_values = &CopyWhat::variants(), case_insensitive = true, help = "What to copy")]
        what: CopyWhat,
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
    let prompt_theme = ColorfulTheme::default();

    let encryption_password = match Password::with_theme(&prompt_theme)
        .with_prompt("Store password")
        .interact()
    {
        Ok(p) => p,
        Err(e) => {
            error!("Could not prompt for password: {}", e);
            process::exit(1);
        }
    };

    if let Some(Subcommand::Init {}) = args.command {
        match util::create_new(&encryption_password) {
            Ok(false) => info!("Store already exists"),
            Ok(true) => info!("Store created"),
            Err(e) => {
                error!("Could not create store: {}", e);
                process::exit(1);
            }
        }
        return;
    }
    let mut entries = util::load_store(&encryption_password);

    match args.command {
        Some(Subcommand::Add {
            name,
            username,
            comments,
        }) => {
            debug!("Adding new entry");
            let password = match Password::with_theme(&prompt_theme)
                .with_prompt("Password")
                .with_confirmation("Confirm password", "Passwords do not match")
                .interact()
            {
                Ok(p) => p,
                Err(e) => {
                    error!("Could not prompt for password: {}", e);
                    process::exit(1);
                }
            };
            entries.push(Entry {
                name,
                username,
                password,
                comments: comments.unwrap_or_default(),
            });
            if let Err(e) = util::write_store(&entries, &encryption_password) {
                error!("Could not save store: {}", e);
                process::exit(1);
            }
            info!("Entry added");
        }
        Some(Subcommand::Search { term }) => {
            if entries.is_empty() {
                info!("Store is empty");
                return;
            }
            let matcher = SkimMatcherV2::default();
            let mut table = Table::new();
            table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
            table.set_titles(row!["Name", "Username", "Comments"]);
            for entry in entries {
                match term.as_ref() {
                    Some(t) => {
                        if matcher.fuzzy_match(&entry.name, t).is_some() {
                            table.add_row(row![entry.name, entry.username, entry.comments]);
                        }
                    }
                    None => {
                        table.add_row(row![entry.name, entry.username, entry.comments]);
                    }
                }
            }
            let match_count = table.row_iter().count();
            debug!("Found {} matching entries", match_count);
            if match_count > 0 {
                table.printstd();
            } else {
                warn!("No matching entries");
            }
        }
        Some(Subcommand::Copy { name, what }) => {
            for entry in entries {
                if entry.name.to_lowercase() == name.to_lowercase() {
                    let mut clipboard: ClipboardContext = match ClipboardProvider::new() {
                        Ok(c) => c,
                        Err(e) => {
                            error!("Could not set up clipboard context: {}", e);
                            process::exit(1);
                        }
                    };
                    let (copy_value, copy_message) = match what {
                        CopyWhat::Username => (entry.username, "username"),
                        CopyWhat::Password => (entry.password, "password"),
                    };
                    if let Err(e) = clipboard.set_contents(copy_value) {
                        error!("Could not copy value to your clipboard: {}", e);
                        process::exit(1);
                    }
                    info!("Copied the {} to your clipboard", copy_message);
                    return;
                }
            }
            warn!("Could not find matching entry");
        }
        Some(Subcommand::Remove { name }) => {
            let start_len = entries.len();
            entries = entries
                .iter()
                .filter(|&entry| entry.name.to_lowercase() != name.to_lowercase())
                .cloned()
                .collect();
            if entries.len() == start_len {
                warn!("could not find matching entry");
            } else {
                if let Err(e) = util::write_store(&entries, &encryption_password) {
                    error!("Could not save store: {}", e);
                    process::exit(1);
                }
                info!("Entry removed");
            }
        }
        _ => {
            error!("Unrecognized subcommand");
            process::exit(1);
        }
    }
}
