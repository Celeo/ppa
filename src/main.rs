use clipboard::{ClipboardContext, ClipboardProvider};
use dialoguer::{theme::ColorfulTheme, Password};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use log::{debug, error, info, warn};
use prettytable::{cell, format, row, Table};
use std::process;
use structopt::StructOpt;

mod util;
use util::{CopyWhat, Entry};

/// Main CLI options;
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

/// CLI subcommands, determining which action to take.
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

/// Configure program logging, the level of which is determined by the debug CLI flag.
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

/// Prompt the user for a password, optionally requiring confirmation and length requirement.
fn prompt_password(confirm: bool, require_length: bool) -> String {
    let prompt_theme = ColorfulTheme::default();
    let mut prompt = Password::with_theme(&prompt_theme);
    prompt.with_prompt("Store password");
    if confirm {
        prompt.with_confirmation("", "");
    }
    loop {
        let encryption_password = match prompt.interact() {
            Ok(p) => p,
            Err(e) => {
                error!("Could not prompt for password: {}", e);
                process::exit(1);
            }
        };
        if !require_length || encryption_password.len() == 32 {
            return encryption_password;
        }
        error!("Password must be 32 characters long");
    }
}

/// Entry point
fn main() {
    let args = Options::from_args();
    setup_logging(args.debug);

    if let Some(Subcommand::Init {}) = args.command {
        match util::store_exists() {
            Ok(true) => info!("Store already exists!"),
            Err(e) => {
                error!("Could not check for store file: {}", e);
                process::exit(1);
            }
            _ => {
                // continue
            }
        }
        let encryption_password = prompt_password(true, true);
        match util::write_store(&vec![], &encryption_password) {
            Ok(()) => info!("Store created"),
            Err(e) => {
                error!("Could not create store: {}", e);
                process::exit(1);
            }
        }
        return;
    }

    let encryption_password = prompt_password(false, true);
    let mut entries = match util::load_store(&encryption_password) {
        Ok(e) => e,
        Err(e) => {
            error!("Could not load store: {}", e);
            process::exit(1);
        }
    };

    match args.command {
        Some(Subcommand::Add {
            name,
            username,
            comments,
        }) => {
            debug!("Adding new entry");
            let password = prompt_password(true, false);
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
