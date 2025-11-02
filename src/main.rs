mod models;
mod formatters;
mod auth;
mod api;
mod workouts;

use clap::{Parser, Subcommand};
use chrono::{NaiveDate, Duration};
use std::collections::HashMap;
use crate::api::ReqwestClient;

#[derive(Parser)]
#[command(name = "wxrust")]
#[command(about = "WeightXReps Rust client")]
struct Args {
    #[arg(short, long, default_value = "credentials.txt")]
    credentials: String,

    #[arg(short = 'a', long = "force-authentication")]
    force_auth: bool,

    #[arg(long, default_value = "auto")]
    color: String,

    #[arg(short, long)]
    verbose: bool,

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

    #[arg(short, long)]
    reverse: bool,

    #[arg(short = 'A', long)]
    all: bool,

    #[arg(short, long)]
    before: Option<String>,

    #[arg(short, long)]
    count: Option<u32>,

    dates: Vec<String>,
}

#[derive(Parser)]
struct ShowArgs {
    #[arg(short, long)]
    summary: bool,

    date: Option<String>,
}

fn parse_date_range(range: &str) -> Result<(NaiveDate, NaiveDate), String> {
    let parts: Vec<&str> = range.split('-').collect();
    if parts.len() == 1 {
        let date = parse_date_boundary(range)?;
        Ok((date, date))
    } else if parts.len() == 2 {
        let start = parse_date_boundary(parts[0])?;
        let end = parse_date_boundary(parts[1])?;
        Ok((start, end))
    } else if parts.len() == 3 {
        let date = parse_date_boundary(range)?;
        Ok((date, date))
    } else {
        Err("Invalid range format".to_string())
    }
}

fn parse_date_boundary(s: &str) -> Result<NaiveDate, String> {
    let normalized = s.replace(".", "-").replace("/", "-");
    if normalized.contains('-') {
        // Assume YYYY-MM-DD
        NaiveDate::parse_from_str(&normalized, "%Y-%m-%d").map_err(|e| format!("Invalid date: {}", e))
    } else {
        if normalized.len() == 4 { // YYYY
            let year = normalized.parse::<i32>().map_err(|_| "Invalid year".to_string())?;
            NaiveDate::from_ymd_opt(year, 1, 1).ok_or("Invalid date".to_string())
        } else if normalized.len() == 6 { // YYYYMM
            let year = normalized[0..4].parse::<i32>().map_err(|_| "Invalid year".to_string())?;
            let month = normalized[4..6].parse::<u32>().map_err(|_| "Invalid month".to_string())?;
            NaiveDate::from_ymd_opt(year, month, 1).ok_or("Invalid date".to_string())
        } else if normalized.len() == 8 { // YYYYMMDD
            let year = normalized[0..4].parse::<i32>().map_err(|_| "Invalid year".to_string())?;
            let month = normalized[4..6].parse::<u32>().map_err(|_| "Invalid month".to_string())?;
            let day = normalized[6..8].parse::<u32>().map_err(|_| "Invalid day".to_string())?;
            NaiveDate::from_ymd_opt(year, month, day).ok_or("Invalid date".to_string())
        } else {
            Err("Invalid date format".to_string())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    unsafe { std::env::set_var("WXRUST_COLOR", &args.color); }

    let home = std::env::var("HOME").unwrap_or(".".to_string());
    let token_path = format!("{}/.config/wxrust/token", home);

    match args.command {
        Commands::List(list) => {
            let mut valid_workouts: HashMap<String, models::JDay> = HashMap::new();
            let dates_to_use = if list.dates.is_empty() {
                let client = ReqwestClient::new_with_verbose(args.verbose);
                let token = match auth::login(&client, &args.credentials, &token_path).await {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                };

                let (from, count) = if list.all {
                    (None, 10000)
                } else if let Some(before) = &list.before {
                    let cnt = list.count.unwrap_or(32);
                    (Some(before.clone()), cnt)
                } else if let Some(cnt) = list.count {
                    (None, cnt)
                } else {
                    (None, 32)
                };

                match workouts::get_dates(&client, &token, from, count, list.reverse).await {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                // Parse ranges
                let mut range_dates = std::collections::HashSet::new();
                for range_str in &list.dates {
                    let (start, end) = match parse_date_range(range_str) {
                        Ok(se) => se,
                        Err(e) => {
                            eprintln!("Invalid date range '{}': {}", range_str, e);
                            std::process::exit(1);
                        }
                    };
                    let mut current = start;
                    while current <= end {
                        range_dates.insert(current.format("%Y-%m-%d").to_string());
                        current += Duration::days(1);
                    }
                }
                let client = ReqwestClient::new_with_verbose(args.verbose);
                let token = match auth::login(&client, &args.credentials, &token_path).await {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                };
                for date in &range_dates {
                    if let Ok(jday) = workouts::get_jday(&client, &token, date).await {
                        valid_workouts.insert(date.clone(), jday);
                    }
                }
                let mut filtered_dates: Vec<String> = valid_workouts.keys().cloned().collect();
                filtered_dates.sort();
                if list.reverse {
                    filtered_dates.reverse();
                }
                filtered_dates
            };

            if dates_to_use.is_empty() {
                eprintln!("No workouts found in the specified range");
                std::process::exit(1);
            }

            if list.details {
                let client = ReqwestClient::new_with_verbose(args.verbose);
                let token = match auth::login(&client, &args.credentials, &token_path).await {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                };
                for date in dates_to_use {
                    let workout = match workouts::get_day(&client, &token, &date).await {
                        Ok(w) => w,
                        Err(e) => {
                            eprintln!("Error getting workout for {}: {}", date, e);
                            continue;
                        }
                    };
                    println!("{}", workout);
                }
            } else if list.summary {
                let client = ReqwestClient::new_with_verbose(args.verbose);
                let token = match auth::login(&client, &args.credentials, &token_path).await {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                };
                for date in dates_to_use {
                    let jday_owned = if let Some(j) = valid_workouts.get(&date) {
                        j.clone()
                    } else {
                        match workouts::get_jday(&client, &token, &date).await {
                            Ok(j) => {
                                let summary = formatters::summarize_workout(&j);
                                println!("{} {}", formatters::color_date(&date), summary);
                                continue;
                            }
                            Err(e) => {
                                eprintln!("Error getting workout for {}: {}", date, e);
                                continue;
                            }
                        }
                    };
                    let summary = formatters::summarize_workout(&jday_owned);
                    println!("{} {}", formatters::color_date(&date), summary);
                }
            } else {
                for date in dates_to_use {
                    println!("{}", date);
                }
            }
        }
        Commands::Show(show) => {
            let client = ReqwestClient::new_with_verbose(args.verbose);
            let token = match auth::login(&client, &args.credentials, &token_path).await {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            };

            let date = if let Some(d) = show.date {
                d
            } else {
                // Show last workout
                let dates = match workouts::get_dates(&client, &token, None, 1, false).await {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                };
                if let Some(d) = dates.get(0) {
                    d.clone()
                } else {
                    eprintln!("No workouts found");
                    std::process::exit(1);
                }
            };

            if show.summary {
                let jday = match workouts::get_jday(&client, &token, &date).await {
                    Ok(j) => j,
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                };
                let summary = formatters::summarize_workout(&jday);
                println!("{} {}", formatters::color_date(&date), summary);
            } else {
                let workout = match workouts::get_day(&client, &token, &date).await {
                    Ok(w) => w,
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                };
                println!("{}", workout);
            }
        }
    }

    Ok(())
}
