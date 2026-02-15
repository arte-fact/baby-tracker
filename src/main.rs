mod db;
mod models;

use std::process;

use chrono::{Local, NaiveDateTime};
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use tabled::{Table, Tabled};

use db::Database;
use models::{Feeding, FeedingType};

#[derive(Parser)]
#[command(name = "baby-tracker")]
#[command(about = "Track baby feeding activity")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new feeding event
    Add {
        /// Baby's name
        #[arg(short, long)]
        name: String,

        /// Feeding type: breast-left (bl), breast-right (br), bottle (b), solid (s)
        #[arg(short = 't', long = "type")]
        feeding_type: String,

        /// Amount in ml (for bottle feeding)
        #[arg(short, long)]
        amount: Option<f64>,

        /// Duration in minutes (for breastfeeding)
        #[arg(short, long)]
        duration: Option<i32>,

        /// Optional notes
        #[arg(long)]
        notes: Option<String>,

        /// Timestamp (YYYY-MM-DD HH:MM format). Defaults to now.
        #[arg(long)]
        time: Option<String>,
    },

    /// List recent feeding events
    List {
        /// Filter by baby name
        #[arg(short, long)]
        name: Option<String>,

        /// Number of entries to show (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Show feeding summary/statistics
    Summary {
        /// Filter by baby name
        #[arg(short, long)]
        name: Option<String>,

        /// Number of days to summarize (default: 1)
        #[arg(short, long, default_value = "1")]
        days: i64,
    },

    /// Delete a feeding event by ID
    Delete {
        /// ID of the feeding event to delete
        id: i64,
    },
}

fn get_db_path() -> std::path::PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "baby-tracker", "baby-tracker") {
        let data_dir = proj_dirs.data_dir();
        std::fs::create_dir_all(data_dir).expect("Failed to create data directory");
        data_dir.join("feedings.db")
    } else {
        // Fallback to current directory
        std::path::PathBuf::from("feedings.db")
    }
}

#[derive(Tabled)]
struct FeedingRow {
    #[tabled(rename = "ID")]
    id: i64,
    #[tabled(rename = "Baby")]
    baby: String,
    #[tabled(rename = "Type")]
    feeding_type: String,
    #[tabled(rename = "Amount (ml)")]
    amount: String,
    #[tabled(rename = "Duration (min)")]
    duration: String,
    #[tabled(rename = "Time")]
    time: String,
    #[tabled(rename = "Notes")]
    notes: String,
}

impl From<&Feeding> for FeedingRow {
    fn from(f: &Feeding) -> Self {
        FeedingRow {
            id: f.id,
            baby: f.baby_name.clone(),
            feeding_type: f.feeding_type.to_string(),
            amount: f
                .amount_ml
                .map(|a| format!("{:.0}", a))
                .unwrap_or_default(),
            duration: f
                .duration_minutes
                .map(|d| d.to_string())
                .unwrap_or_default(),
            time: f.timestamp.format("%Y-%m-%d %H:%M").to_string(),
            notes: f
                .notes
                .as_deref()
                .unwrap_or("")
                .chars()
                .take(30)
                .collect(),
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let db_path = get_db_path();
    let db = match Database::open(&db_path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Error opening database: {}", e);
            process::exit(1);
        }
    };

    match cli.command {
        Commands::Add {
            name,
            feeding_type,
            amount,
            duration,
            notes,
            time,
        } => {
            let ft = match FeedingType::from_str(&feeding_type) {
                Ok(ft) => ft,
                Err(e) => {
                    eprintln!("{}", e);
                    process::exit(1);
                }
            };

            let timestamp = match time {
                Some(t) => {
                    match NaiveDateTime::parse_from_str(&t, "%Y-%m-%d %H:%M") {
                        Ok(ts) => ts,
                        Err(_) => {
                            eprintln!("Invalid time format. Use: YYYY-MM-DD HH:MM");
                            process::exit(1);
                        }
                    }
                }
                None => Local::now().naive_local(),
            };

            let feeding = Feeding {
                id: 0,
                baby_name: name,
                feeding_type: ft,
                amount_ml: amount,
                duration_minutes: duration,
                notes,
                timestamp,
            };

            match db.add_feeding(&feeding) {
                Ok(id) => {
                    println!(
                        "Added feeding #{} for {} ({}) at {}",
                        id,
                        feeding.baby_name,
                        feeding.feeding_type,
                        feeding.timestamp.format("%Y-%m-%d %H:%M")
                    );
                }
                Err(e) => {
                    eprintln!("Error adding feeding: {}", e);
                    process::exit(1);
                }
            }
        }

        Commands::List { name, limit } => {
            let feedings = match db.list_feedings(name.as_deref(), limit) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Error listing feedings: {}", e);
                    process::exit(1);
                }
            };

            if feedings.is_empty() {
                println!("No feeding events found.");
                return;
            }

            let rows: Vec<FeedingRow> = feedings.iter().map(FeedingRow::from).collect();
            let table = Table::new(rows).to_string();
            println!("{}", table);
        }

        Commands::Summary { name, days } => {
            let summary = match db.get_summary(name.as_deref(), days) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error generating summary: {}", e);
                    process::exit(1);
                }
            };

            let period = if summary.days == 1 {
                "today".to_string()
            } else {
                format!("last {} days", summary.days)
            };

            println!("=== Feeding Summary ({}) ===", period);
            if let Some(ref n) = name {
                println!("Baby: {}", n);
            }
            println!("Total feedings: {}", summary.total_feedings);
            if summary.total_ml > 0.0 {
                println!("Total volume: {:.0} ml", summary.total_ml);
            }
            if summary.total_minutes > 0 {
                println!("Total nursing time: {} min", summary.total_minutes);
            }
            println!();
            if !summary.by_type.is_empty() {
                println!("By type:");
                for (ft, count) in &summary.by_type {
                    println!("  {}: {}", ft, count);
                }
            }
        }

        Commands::Delete { id } => match db.delete_feeding(id) {
            Ok(true) => println!("Deleted feeding #{}.", id),
            Ok(false) => {
                eprintln!("No feeding found with ID {}.", id);
                process::exit(1);
            }
            Err(e) => {
                eprintln!("Error deleting feeding: {}", e);
                process::exit(1);
            }
        },
    }
}
