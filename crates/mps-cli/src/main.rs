use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use std::fs;

#[derive(Parser)]
#[command(
    name = "mps",
    version,
    about = "Memellow Programming System - deterministic Pilates class plan generator"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a SQLite database with the MPS schema.
    InitDb {
        /// Path to SQLite database file.
        #[arg(long)]
        database: String,
    },
    /// Load seed SQL into a SQLite database.
    Seed {
        /// Path to SQLite database file.
        #[arg(long)]
        database: String,
        /// Path to seed SQL file.
        #[arg(long)]
        file: String,
    },
    /// Generate a class plan from a JSON request.
    Generate {
        /// Path to ClassRequest JSON file.
        input: String,
        /// Path to SQLite database file.
        #[arg(long)]
        database: String,
        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Markdown)]
        format: OutputFormat,
        /// Optional file path. If omitted, output is written to stdout.
        #[arg(long)]
        output: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Markdown,
    Md,
    Json,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::InitDb { database } => {
            let _repo = mps_db::SqliteRepository::open(&database)
                .context("failed to initialize database")?;
            println!("Database initialized: {database}");
        }
        Commands::Seed { database, file } => {
            let sql = fs::read_to_string(&file)
                .with_context(|| format!("failed to read seed file: {file}"))?;
            let repo =
                mps_db::SqliteRepository::open(&database).context("failed to open database")?;
            repo.load_seed(&sql).context("failed to load seed data")?;
            println!("Seed data loaded from: {file}");
        }
        Commands::Generate {
            input,
            database,
            format,
            output,
        } => {
            let json = fs::read_to_string(&input)
                .with_context(|| format!("failed to read input file: {input}"))?;
            let request: mps_core::ClassRequest =
                serde_json::from_str(&json).context("failed to parse ClassRequest JSON")?;

            let repo =
                mps_db::SqliteRepository::open(&database).context("failed to open database")?;
            let plan = mps_core::generate_class_plan(&request, &repo)
                .context("failed to generate class plan")?;

            let rendered = match format {
                OutputFormat::Json => serde_json::to_string_pretty(&plan)?,
                OutputFormat::Markdown | OutputFormat::Md => mps_core::render_markdown(&plan),
            };

            if let Some(path) = output {
                fs::write(&path, rendered)
                    .with_context(|| format!("failed to write output file: {path}"))?;
            } else {
                println!("{rendered}");
            }
        }
    }

    Ok(())
}
