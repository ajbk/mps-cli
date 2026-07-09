use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;

#[derive(Parser)]
#[command(name = "mps", version, about = "Memellow Programming System — Pilates class plan generator")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize database with schema
    InitDb {
        /// Path to SQLite database file
        #[arg(long)]
        database: String,
    },
    /// Load seed data into database
    Seed {
        /// Path to SQLite database file
        #[arg(long)]
        database: String,
        /// Path to seed SQL file
        #[arg(long)]
        file: String,
    },
    /// Generate a class plan
    Generate {
        /// Path to JSON request file
        input: String,
        /// Path to SQLite database file
        #[arg(long)]
        database: String,
        /// Output format: markdown or json
        #[arg(long, default_value = "markdown")]
        format: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::InitDb { database } => {
            let _repo = mps_db::SqliteRepository::open(&database)
                .context("Failed to initialize database")?;
            println!("Database initialized: {}", database);
        }
        Commands::Seed { database, file } => {
            let sql = fs::read_to_string(&file)
                .with_context(|| format!("Failed to read seed file: {}", file))?;
            let repo = mps_db::SqliteRepository::open(&database)
                .context("Failed to open database")?;
            repo.load_seed(&sql)
                .context("Failed to load seed data")?;
            println!("Seed data loaded from: {}", file);
        }
        Commands::Generate {
            input,
            database,
            format,
        } => {
            let json = fs::read_to_string(&input)
                .with_context(|| format!("Failed to read input file: {}", input))?;
            let request: mps_core::ClassRequest =
                serde_json::from_str(&json).context("Failed to parse ClassRequest JSON")?;

            let repo = mps_db::SqliteRepository::open(&database)
                .context("Failed to open database")?;

            let plan = mps_core::generate_class_plan(&request, &repo)
                .context("Failed to generate class plan")?;

            match format.as_str() {
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&plan)?);
                }
                _ => {
                    println!("{}", mps_core::render_markdown(&plan));
                }
            }
        }
    }

    Ok(())
}
