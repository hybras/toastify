use clap::{ArgEnum, Parser, Subcommand};
use std::{num::NonZeroU32, path::PathBuf, str::FromStr};
use notify_rust::{Hint, Notification, Urgency};

#[derive(ArgEnum, Clone, Copy)]
pub enum UrgencyShim {
    Low,
    Normal,
    Critical,
}

impl From<UrgencyShim> for Urgency {
    fn from(urgency: UrgencyShim) -> Urgency {
        match urgency {
            UrgencyShim::Low => Urgency::Low,
            UrgencyShim::Normal => Urgency::Normal,
            UrgencyShim::Critical => Urgency::Critical,
        }
    }
}

struct HintShim(Hint);

impl FromStr for HintShim {
    type Err = String;

    fn from_str(pattern: &str) -> Result<Self, Self::Err> {
        let parts = pattern.split(':').collect::<Vec<&str>>();
        if parts.len() != 3 {
            return Err("Wrong number of segments".into());
        }
        let (_typ, name, value) = (parts[0], parts[1], parts[2]);
        Hint::from_key_val(name, value).map(|it| HintShim(it))
    }
}
#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Starts a little notification server for testing
    #[cfg(all(unix, not(target_os = "macos")))]
    Server,
    /// Shows information about the running notification server
    #[cfg(all(unix, not(target_os = "macos")))]
    Info,
    /// Shows a notification
    Send {
        /// Title of the Notification.
        title: String,
        /// Message body
        body: Option<String>,
        /// Set a specific app-name manually.
        #[clap(short, long)]
        app_name: Option<String>,
        /// Time until expiration in milliseconds.
        #[clap(short = 't', long)]
        expire_time: Option<NonZeroU32>,
        /// Icon of notification.
        #[clap(short = 'i', long)]
        icon: Option<PathBuf>,
        /// Specifies the ID and overrides existing notifications with the same ID.
        id: Option<u32>, // TODO: Type is u32 or string?
        /// Set a category.
        #[clap(short, long)]
        category: Option<String>,
        /// Specifies basic extra data to pass. Valid types are int, double, string and byte. Pattern: TYPE:NAME:VALUE
        #[clap(long)]
        hint: Option<HintShim>,
        /// How urgent is it.
        #[clap(short, long, arg_enum)]
        urgency: Option<UrgencyShim>,
        /// Also prints notification to stdout
        #[clap(short, long)]
        debug: bool,
    },
}

fn main() {
    let args = Cli::parse();
}
