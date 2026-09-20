use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "fcard",
    version,
    about = "A TUI flashcard with SM-2 spaced repitioin",
    long_about = None
)]
pub struct Cli {
    pub decks: Vec<PathBuf>,
    #[arg(long, short = 'p', value_name = "PATH")]
    pub path: Option<PathBuf>,
    #[arg(long, short = 't', value_name = "THEME")]
    pub theme: Option<String>,
    #[command(subcommand)]
    pub command: Option<Commands>,
}
#[derive(Subcommand, Debug)]
pub enum Commands {
    Init { path: PathBuf },
    Edit { path: PathBuf },
}
