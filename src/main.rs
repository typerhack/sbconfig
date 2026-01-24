// src/main.rs
// sbconfig - TUI tool for managing sing-box SSH proxy configurations

mod db;
mod error;
mod singbox;
mod ssh;
mod ui;
mod utils;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// sbconfig - sing-box SSH Proxy Manager
#[derive(Parser)]
#[command(name = "sbconfig")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Database file path
    #[arg(short, long, default_value = "/var/lib/sbconfig/sbconfig.db")]
    database: PathBuf,

    /// Run in development mode (uses localhost)
    #[arg(long)]
    dev: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the TUI interface (default)
    Tui,

    /// Show sing-box status
    Status,

    /// List all users
    Users,

    /// Show version information
    Version,
}

fn main() {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let cli = Cli::parse();

    // Run the appropriate command
    let result = match cli.command {
        Some(Commands::Status) => run_status(),
        Some(Commands::Users) => run_users(&cli.database),
        Some(Commands::Version) => run_version(),
        Some(Commands::Tui) | None => run_tui(&cli.database, cli.dev),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

/// Run the TUI interface
fn run_tui(db_path: &PathBuf, dev_mode: bool) -> error::Result<()> {
    // Ensure database directory exists
    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Open database
    let db = db::Database::open(db_path)?;

    // Set development mode in settings if specified
    if dev_mode {
        db.set_setting("mode", "development")?;
    }

    // Initialize terminal
    let mut terminal = ui::init()?;

    // Create and run app
    let mut app = ui::App::new(db);
    let result = app.run(&mut terminal);

    // Restore terminal
    ui::restore()?;

    result
}

/// Show sing-box status
fn run_status() -> error::Result<()> {
    let info = singbox::detect_singbox()?;

    println!("sing-box Status");
    println!("---------------");

    if info.installed {
        println!("Status:   Installed");
        if let Some(path) = info.path {
            println!("Path:     {}", path);
        }
        if let Some(version) = info.version {
            println!("Version:  {}", version);
        }
    } else {
        println!("Status:   Not installed");
        println!();
        println!("{}", singbox::get_install_instructions());
    }

    Ok(())
}

/// List all users
fn run_users(db_path: &PathBuf) -> error::Result<()> {
    if !db_path.exists() {
        println!("Database not found. Run sbconfig first to initialize.");
        return Ok(());
    }

    let db = db::Database::open(db_path)?;
    let users = db.list_users()?;

    if users.is_empty() {
        println!("No users configured.");
        return Ok(());
    }

    println!("Configured Users");
    println!("----------------");

    for user in users {
        let status = if user.is_active { "Active" } else { "Disabled" };
        println!(
            "  {} ({}) - {} - {}",
            user.username, user.key_type, status, user.created_at
        );
    }

    Ok(())
}

/// Show version information
fn run_version() -> error::Result<()> {
    println!("sbconfig {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("sing-box SSH Proxy Manager");
    println!("https://github.com/typerhack/sbconfig");

    Ok(())
}
