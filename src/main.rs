mod models;
mod formatters;
mod auth;
mod api;
mod workouts;

use clap::{Parser, Subcommand};
use chrono::{NaiveDate, Datelike};
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
    let parts: Vec<&str> = range.split("..").collect();
    if parts.len() == 1 {
        let start = parse_date_boundary(range, false)?;
        let end = parse_date_boundary(range, true)?;
        Ok((start, end))
    } else if parts.len() == 2 {
        let start = parse_date_boundary(parts[0], false)?;
        let end = parse_date_boundary(parts[1], true)?;
        Ok((start, end))
    } else {
        Err("Invalid range format".to_string())
    }
}

// generate a string from text given.
// looks for dates like YYYYMMDD, YYYY/MM/DD, YYYY-MM-DD, or YYYY.MM.DD
// if DD is missing picks the first or last day of the month (depending on end)
// if MMDD is missing picks the first or last day of the year (depending on end)
// examples:
// "20250527"             -> NativeDate of 2025/05/27       (ignores end)
// "20250-05" end=false   -> NativeDate of 2025/05/01
// "20250-05" end=true    -> NativeDate of 2025/05/31
fn parse_date_boundary(s: &str, end: bool) -> Result<NaiveDate, String> {
    let mut parts = Vec::new();
    for p in s.split(['-', '/', '.']) {
        if !p.is_empty() {
            parts.push(p);
        }
    }

    let (year_str, month_str, day_str) = if parts.len() == 1 {
        let compact = parts[0];
        if compact.len() == 8 {
            // YYYYMMDD
            (compact[0..4].to_string(), compact[4..6].to_string(), compact[6..8].to_string())
        } else if compact.len() == 6 {
            // YYYYMM
            (compact[0..4].to_string(), compact[4..6].to_string(), "".to_string())
        } else if compact.len() == 4 {
            // YYYY
            (compact.to_string(), "".to_string(), "".to_string())
        } else {
            return Err("Invalid compact date format".to_string());
        }
    } else if parts.len() == 2 {
        (parts[0].to_string(), parts[1].to_string(), "".to_string())
    } else if parts.len() == 3 {
        (parts[0].to_string(), parts[1].to_string(), parts[2].to_string())
    } else if parts.len() == 0 {
        return Err("Empty date string".to_string());
    } else {
        return Err("Too many parts".to_string());
    };

    let year: i32 = year_str.parse().map_err(|_| "Invalid year")?;
    if year_str.len() != 4 {
        return Err("Year must be 4 digits".to_string());
    }

    if month_str.is_empty() {
        return Ok(if end {
            NaiveDate::from_ymd_opt(year, 12, 31).unwrap()
        } else {
            NaiveDate::from_ymd_opt(year, 1, 1).unwrap()
        });
    }

    let month: u32 = month_str.parse().map_err(|_| "Invalid month")?;
    if month == 0 || month > 12 {
        return Err("Invalid month".to_string());
    }

    if day_str.is_empty() {
        let last_day = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap() - chrono::Duration::days(1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap() - chrono::Duration::days(1)
        }.day();

        let day = if end { last_day } else { 1 };
        return Ok(NaiveDate::from_ymd_opt(year, month, day).unwrap());
    }

    let day: u32 = day_str.parse().map_err(|_| "Invalid day")?;
    NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| "Invalid date".to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    unsafe { std::env::set_var("WXRUST_COLOR", &args.color); }

    let home = std::env::var("HOME").unwrap_or(".".to_string());
    let token_path = format!("{}/.config/wxrust/token", home);

    match args.command {
        Commands::List(list) => {
            let valid_workouts: HashMap<String, models::JDay> = HashMap::new();
            let client = ReqwestClient::new_with_verbose(args.verbose);
            let token = match auth::login(&client, &args.credentials, &token_path).await {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            };
            let dates_to_use = if list.dates.is_empty() {
                let (latest, oldest, count) = if list.all {
                    (None, None, 10000)
                } else if let Some(before) = &list.before {
                    let cnt = list.count.unwrap_or(32);
                    (Some(before.clone()), None, cnt)
                } else if let Some(cnt) = list.count {
                    (None, None, cnt)
                } else {
                    (None, None, 32)
                };

                match workouts::get_dates(&client, &token, latest, oldest, count, list.reverse).await {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                // Parse ranges
                let mut all_dates: Vec<String> = vec![];
                for range_str in &list.dates {
                    let (oldest, latest) = match parse_date_range(range_str) {
                        Ok(start_end) => start_end,
                        Err(e) => {
                            eprintln!("Invalid date range '{}': {}", range_str, e);
                            std::process::exit(1);
                        }
                    };
                    let count = ((oldest - latest).num_days().abs() + 1) as u32;
                    let dates = match workouts::get_dates(&client, &token,
                                        Some(latest.to_string()), Some(oldest.to_string()), count, false).await {
                        Ok(d) => d,
                        Err(e) => {
                            eprintln!("{}", e);
                            std::process::exit(1);
                        }
                    };
                    all_dates.extend(dates);
                }
                all_dates.sort();
                if list.reverse {
                    all_dates.reverse();
                }
                all_dates
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
                let dates = match workouts::get_dates(&client, &token, None, None, 1, false).await {
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
