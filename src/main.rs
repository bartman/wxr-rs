mod models;
mod formatters;
mod auth;
mod api;
mod workouts;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wxrust")]
#[command(about = "WeightXReps Rust client")]
struct Args {
    #[arg(short, long, default_value = "credentials.txt")]
    credentials: String,

    #[arg(short = 'a', long = "force-authentication")]
    force_auth: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    List(ListArgs),
    Show(ShowArgs),
}

#[derive(Parser)]
struct ListArgs {
    #[arg(short, long)]
    details: bool,

    #[arg(short, long)]
    summary: bool,

    #[arg(long)]
    before: Option<String>,

    #[arg(long)]
    count: Option<u32>,

    #[arg(short, long)]
    all: bool,

    dates: Vec<String>,
}

#[derive(Parser)]
struct ShowArgs {
    #[arg(short, long)]
    summary: bool,

    date: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let home = std::env::var("HOME").unwrap_or(".".to_string());
    let token_path = format!("{}/.config/wxrust/token", home);

    match args.command {
        Commands::List(list) => {
            let token = match auth::login(&args.credentials, &token_path).await {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            };

            let (from, count) = if let Some(before) = &list.before {
                let cnt = list.count.unwrap_or(32);
                (Some(before.clone()), cnt)
            } else if let Some(count_str) = list.dates.get(0) {
                let cnt = count_str.parse().unwrap_or(32);
                (None, cnt)
            } else {
                (None, 32)
            };

            let dates = match workouts::get_dates_from(&token, from, count).await {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            };

            for date in dates {
                println!("{}", date);
            }
        }
        Commands::Show(show) => {
            let token = match auth::login(&args.credentials, &token_path).await {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            };

            let workout = match workouts::get_day(&token, &show.date).await {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            };

            println!("{}", workout);
        }
    }

    Ok(())
}
