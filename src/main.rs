use clap::{Args, Parser};
use maildir::Maildir;
use std::error::Error;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[clap(about = "A rewrite of the old Python 2 archivemail command with less features")]
#[clap(author, version, long_about = None)]
struct Cli {
    #[clap(value_parser)]
    mailbox: String,
    #[command(flatten)]
    action: Action,
    #[clap(
        short,
        long,
        value_parser,
        default_value_t = 31,
        help = "Archive messages older than NUMBER of days",
        value_name = "NUMBER"
    )]
    days: u64,
    #[clap(
        short = 'n',
        long,
        action,
        help = "Don't perform any actions, only print which emails would be affected",
        default_value_t = false
    )]
    dry_run: bool,
    #[clap(
        short,
        long,
        action,
        help = "Print extra information",
        default_value_t = false
    )]
    verbose: bool,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct Action {
    #[clap(
        short,
        long,
        value_parser,
        help = "Location of output maildir",
        group = "input"
    )]
    output: Option<String>,
    #[clap(
        long,
        action,
        help = "Delete mails instead of archiving them",
        default_value_t = false,
        conflicts_with = "output",
        group = "input"
    )]
    delete: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let log_level = if cli.verbose || cli.dry_run {
        log::LevelFilter::Info
    } else {
        log::LevelFilter::Error
    };

    simple_logger::SimpleLogger::new()
        .with_level(log_level)
        .init()
        .unwrap();

    let maildir = Maildir::from(cli.mailbox);
    let dest = Maildir::from(cli.action.output.unwrap_or("".to_string()));

    let days = Duration::from_secs(60 * 60 * 24 * cli.days);
    let older = SystemTime::now()
        .checked_sub(days)
        .unwrap()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64; // this seems the least of two evils compared to using chrono

    for mut mail in maildir.list_cur().filter_map(Result::ok) {
        match mail.date() {
            Ok(date_of_mail) if date_of_mail < older => {
                let mail_id = mail.id();
                log::info!("Processing mail {:?}", mail_id);

                if !cli.dry_run {
                    let result = if cli.action.delete {
                        maildir.delete(mail_id)
                    } else {
                        maildir.move_to(mail_id, &dest)
                    };

                    match result {
                        Ok(_) => log::info!("Processed mail {:?}", mail_id),
                        Err(error) => log::error!(
                            "Failed to process mail {:?}, because {:?}",
                            mail_id,
                            error
                        ),
                    }
                }
            }
            Ok(_) => {} // Used as a catch all for mails that are newer than the set date
            Err(error) => {
                log::error!(
                    "Encountered an error while proccessing mail {:?}. Error is {:?}",
                    mail.id(),
                    error
                );
            }
        };
    }
    Ok(())
}
