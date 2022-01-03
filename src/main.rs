use clap::{ArgEnum, Parser, Subcommand};
use notify_rust::{error::Result as nResult, Hint, Notification, Urgency};

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

fn parse_hint(pattern: &str) -> Result<Hint, String> {
    let parts = pattern.split(':').collect::<Vec<&str>>();
    if parts.len() != 3 {
        return Err("Wrong number of segments".into());
    }
    let (_typ, name, value) = (parts[0], parts[1], parts[2]);
    Hint::from_key_val(name, value)
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
        #[cfg(all(unix, not(target_os = "macos")))]
        #[clap(flatten)]
        linux_args: LinuxArgs,
    },
}

#[cfg(all(unix, not(target_os = "macos")))]
#[derive(clap::Args)]
struct LinuxArgs {
    /// Time until expiration in milliseconds.
    #[clap(short = 't', long)]
    expire_time: Option<i32>,
    /// Icon of notification.
    #[clap(short = 'i', long)]
    icon: Option<std::path::PathBuf>,
    /// Specifies the ID and overrides existing notifications with the same ID.
    id: Option<u32>, // TODO: Type is u32 or string?
    /// Set a category.
    #[clap(short, long)]
    categories: Option<Vec<String>>,
    /// Specifies basic extra data to pass. Valid types are int, double, string and byte. Pattern: TYPE:NAME:VALUE
    #[clap(long, parse(try_from_str = parse_hint))]
    hint: Option<Hint>,
    /// How urgent is it.
    #[clap(short, long, arg_enum)]
    urgency: Option<UrgencyShim>,
    /// Also prints notification to stdout
    #[clap(short, long)]
    debug: bool,
}

fn main() -> nResult<()> {
    let args = Cli::parse();

    match args.command {
        #[cfg(all(unix, not(target_os = "macos")))]
        Commands::Server => {
            use notify_rust::server::NotificationServer;
            use std::thread;
            let server = NotificationServer::create();
            thread::spawn(move || {
                NotificationServer::start(&server, |notification| println!("{:#?}", notification))
            });

            println!("Press enter to exit.\n");

            std::thread::sleep(std::time::Duration::from_millis(1_000));

            Notification::new()
                .summary("Notification Logger")
                .body("If you can read this in the console, the server works fine.")
                .show()
                .expect("Was not able to send initial test message");

            let mut _devnull = String::new();
            let _ = std::io::stdin().read_line(&mut _devnull);
            println!("Thank you for choosing toastify.");
        }
        #[cfg(all(unix, not(target_os = "macos")))]
        Commands::Info => {
            match notify_rust::get_server_information() {
                Ok(info) => println!("server information:\n {:?}\n", info),
                Err(error) => eprintln!("{}", error),
            }

            match notify_rust::get_capabilities() {
                Ok(caps) => println!("capabilities:\n {:?}\n", caps),
                Err(error) => eprintln!("{}", error),
            }
        }
        Commands::Send {
            title,
            body,
            app_name,
            #[cfg(all(unix, not(target_os = "macos")))]
            linux_args,
        } => {
            let mut notification = Notification::new();

            notification.summary(&title);

            if let Some(body) = body {
                notification.body(&body);
            }

            if let Some(appname) = app_name {
                notification.appname(&appname);
            }

            #[cfg(all(unix, not(target_os = "macos")))]
            {
                let LinuxArgs {
                    expire_time,
                    icon,
                    id,
                    categories,
                    hint,
                    urgency,
                    debug,
                } = linux_args;
                if let Some(id) = id {
                    notification.id(id);
                }

                if let Some(icon) = icon {
                    notification.icon(icon.to_str().expect("Icon path is not valid unicode"));
                }

                if let Some(timeout) = expire_time {
                    notification.timeout(timeout);
                }

                if let Some(urgency) = urgency {
                    notification.urgency(urgency.into())
                }

                if let Some(hint) = hint {
                    notification.hint(hint);
                }

                if let Some(categories) = category {
                    for category in categories {
                        notification.hint(Hint::Category(category));
                    }
                }

                if debug {
                    notification.show_debug()
                } else {
                    notification.show()
                }
                .map(|_| ())
            }

            #[cfg(target_os = "macos")]
            notification.show().map(|_| ())
        }
    }
}
