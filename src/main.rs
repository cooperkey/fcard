mod cli;
mod engine;
mod models;
mod storage;
mod ui;

use crate::cli::{Cli, Commands};
use crate::models::theme::{Theme, ThemeName};
use crate::storage::config_store::ConfigStore;
use crate::storage::deck_loader::{detect_format, load_deck, write_template};
use crate::storage::stats_store::StatsStore;
use crate::ui::app::{App, AppMode};
use clap::Parser;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::stdout;
use std::path::PathBuf;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Some(command) = &cli.command {
        match command {
            Commands::Init { path } => {
                let format = detect_format(path)?;
                write_template(path, format)?;
                println!("create new deck template at {:?}", path);
                return Ok(());
            }
            Commands::Edit { path } => {
                let editor = std::env::var("EDITOR")
                    .or_else(|_| std::env::var("VISUAL"))
                    .unwrap_or_else(|_| "nvim".to_string());
                let path_str = path.to_string_lossy();
                let shell_cmd = format!("{} {}", editor, path_str);
                std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&shell_cmd)
                    .status()?;
                return Ok(());
            }
        }
    }

    let mut config_store = ConfigStore::load().unwrap_or_else(|_| {
        eprintln!("Failed to load config, starting with defaults");
        ConfigStore {
            config: Default::default(),
            path: PathBuf::from("config.json"),
        }
    });

    let resolved_theme_name = if let Some(cli_theme) = &cli.theme {
        ThemeName::from_cli_str(cli_theme).unwrap_or_else(|| {
            eprintln!("warning: unrecognized theme '{}', using default", cli_theme);
            config_store.config.theme
        })
    } else {
        config_store.config.theme
    };

    let theme = Theme::from_name(resolved_theme_name);
    let mut stats_store = StatsStore::load().unwrap_or_else(|_| {
        eprintln!("Failed to load stats, starting with none");
        StatsStore {
            states: Default::default(),
            path: PathBuf::from("stats.json"),
        }
    });

    let mut search_paths = Vec::new();
    if let Some(p) = &cli.path {
        search_paths.push(p.clone());
    }
    for p in &cli.decks {
        search_paths.push(p.clone());
    }
    if search_paths.is_empty() {
        search_paths.push(PathBuf::from("."));
    }

    let mut explicit_files = Vec::new();
    let mut candidate_paths = Vec::new();
    for p in &search_paths {
        if p.is_file() {
            if detect_format(p).is_ok() {
                explicit_files.push(p.clone());
            }
        } else if p.is_dir() {
            scan_directory_recursive(p, &mut candidate_paths);
        }
    }

    let mut target_paths = explicit_files;
    let marked_paths: Vec<std::path::PathBuf> = candidate_paths
        .into_iter()
        .filter(|p| storage::deck_loader::has_fcard_marker(p))
        .collect();
    target_paths.extend(marked_paths);
    let mut decks = Vec::new();
    for path in &target_paths {
        match load_deck(path) {
            Ok(deck) => decks.push(deck),
            Err(e) => {
                eprintln!("error loading deck {:?}: {}", path, e);
            }
        }
    }

    let mut app = App::new(decks, theme);
    app.deck_positions = config_store.config.deck_positions.clone();
    if let Some(deck) = app.current_deck() {
        let key = App::deck_key(deck);
        let saved = app.deck_positions.get(&key).copied().unwrap_or(0);
        app.selected_card = saved.min(deck.cards.len().saturating_sub(1));
    }
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    loop {
        if app.needs_clear {
            terminal.clear()?;
            app.needs_clear = false;
        }

        terminal.draw(|f| {
            let area = f.area();
            let panes = ui::layout::compute_layout(area);

            ui::deck_pane::draw(f, panes.deck_list, &app, &stats_store);
            ui::card_list_pane::draw(f, panes.card_list, &mut app, &stats_store);
            ui::stats_pane::draw(f, panes.stats, &app, &stats_store);

            match app.mode {
                AppMode::Browse => {
                    ui::preview_pane::draw(f, panes.main_view, &app);
                }
                AppMode::Flashcard => {
                    ui::flashcard_mode::draw(f, panes.main_view, &app);
                }
                AppMode::Learn => {
                    ui::learn_mode::draw(f, panes.main_view, &app);
                }
                AppMode::SpacedRep => {
                    ui::spaced_rep_mode::draw(f, panes.main_view, &app, &stats_store);
                }
                AppMode::ThemePicker => {
                    ui::preview_pane::draw(f, panes.main_view, &app);
                    ui::theme_picker::draw(f, area, &app);
                }
                AppMode::Command => {
                    ui::preview_pane::draw(f, panes.main_view, &app);
                }
                AppMode::AddCard => {
                    ui::preview_pane::draw(f, panes.main_view, &app);
                    ui::add_card_popup::draw(f, area, &app);
                }
                AppMode::Search => {
                    ui::preview_pane::draw(f, panes.main_view, &app);
                }
            }
            ui::keybindings::draw(f, panes.status_bar, &app);
        })?;

        if let Ok(Some(crossterm::event::Event::Key(key_event))) =
            ui::event::poll_event(Duration::from_millis(100))
        {
            app.clear_status();
            ui::event::handle_key(&mut app, key_event, &mut stats_store, &mut config_store);
        }
        if app.should_quit {
            break;
        }
    }

    app.save_current_position();
    config_store.config.deck_positions = app.deck_positions.clone();
    let _ = config_store.save();
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn scan_directory_recursive(dir: &std::path::Path, candidates: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if detect_format(&path).is_ok() {
                    candidates.push(path);
                }
            } else if path.is_dir() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !file_name.starts_with('.') {
                    scan_directory_recursive(&path, candidates);
                }
            }
        }
    }
}
