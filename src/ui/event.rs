use crate::engine::matcher;
use crate::engine::sm2;
use crate::models::theme::ThemeName;
use crate::storage::config_store::ConfigStore;
use crate::storage::stats_store::StatsStore;
use crate::ui::app::{AddCardFocus, App, AppMode, FocusedPane, InputEditMode};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

pub fn poll_event(timeout: Duration) -> std::io::Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}

pub fn handle_key(app: &mut App, key: KeyEvent, stats: &mut StatsStore, config: &mut ConfigStore) {
    if key.modifiers == KeyModifiers::CONTROL {
        if let KeyCode::Char('c') = key.code {
            app.should_quit = true;
            return;
        }
    }
    match app.mode {
        AppMode::Browse => handle_browse(app, key, stats, config),
        AppMode::Flashcard => handle_flashcard(app, key, stats, config),
        AppMode::Learn => handle_learn(app, key, stats),
        AppMode::SpacedRep => handle_spaced_rep(app, key, stats, config),
        AppMode::ThemePicker => handle_theme_picker(app, key, config),
        AppMode::Command => handle_command(app, key, stats, config),
        AppMode::AddCard => handle_add_card(app, key),
        AppMode::Search => handle_search(app, key),
    }
}

fn cycle_pane_next(pane: FocusedPane) -> FocusedPane {
    match pane {
        FocusedPane::DeckList => FocusedPane::CardList,
        FocusedPane::CardList => FocusedPane::MainView,
        FocusedPane::MainView => FocusedPane::Stats,
        FocusedPane::Stats => FocusedPane::DeckList,
    }
}

fn cycle_pane_prev(pane: FocusedPane) -> FocusedPane {
    match pane {
        FocusedPane::DeckList => FocusedPane::Stats,
        FocusedPane::CardList => FocusedPane::DeckList,
        FocusedPane::MainView => FocusedPane::CardList,
        FocusedPane::Stats => FocusedPane::MainView,
    }
}

fn handle_browse(app: &mut App, key: KeyEvent, stats: &mut StatsStore, config: &mut ConfigStore) {
    if key.code != KeyCode::Esc {
        app.search_esc_count = 0;
    }
    if app.pending_g {
        app.pending_g = false;
        if key.code == KeyCode::Char('g') {
            app.jump_to_first();
            return;
        }
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('d') => {
                app.scroll_half_page_down();
                return;
            }
            KeyCode::Char('u') => {
                app.scroll_half_page_up();
                return;
            }
            _ => {}
        }
    }

    match key.code {
        KeyCode::Esc => {
            if !app.search_query.is_empty() || !app.search_results.is_empty() {
                app.search_esc_count += 1;
                if app.search_esc_count >= 2 {
                    app.clear_search();
                } else {
                    app.set_status("press <esc> again to clear search");
                }
            }
        }

        KeyCode::Char('q') => {
            config.config.deck_positions = app.deck_positions.clone();
            let _ = stats.save();
            let _ = config.save();
            app.should_quit = true;
        }
        KeyCode::Char(':') => {
            app.mode = AppMode::Command;
            app.command_buffer = ":".to_string();
            app.command_cursor = 1;
        }
        KeyCode::Char('/') => {
            app.mode = AppMode::Search;
            app.search_buffer.clear();
        }
        KeyCode::Char('n') => app.next_search_match(),
        KeyCode::Char('N') => app.prev_search_match(),
        KeyCode::Char('g') => app.pending_g = true,
        KeyCode::Char('G') => app.jump_to_last(),
        KeyCode::Char('H') => app.jump_to_first(),
        KeyCode::Char('M') => app.jump_to_middle(),
        KeyCode::Up | KeyCode::Char('k') => match app.focused_pane {
            FocusedPane::DeckList => app.prev_deck(),
            FocusedPane::CardList | FocusedPane::MainView => app.prev_card(),
            FocusedPane::Stats => {}
        },
        KeyCode::Down | KeyCode::Char('j') => match app.focused_pane {
            FocusedPane::DeckList => app.next_deck(),
            FocusedPane::CardList | FocusedPane::MainView => app.next_card(),
            FocusedPane::Stats => {}
        },
        KeyCode::Tab | KeyCode::Right => {
            app.focused_pane = cycle_pane_next(app.focused_pane);
        }
        KeyCode::BackTab | KeyCode::Left => {
            app.focused_pane = cycle_pane_prev(app.focused_pane);
        }
        KeyCode::Enter => {
            if app.current_deck().is_some() {
                app.mode = AppMode::Flashcard;
                app.card_flipped = false;
            }
        }
        KeyCode::Char('l') => {
            if app
                .current_deck()
                .map(|d| !d.cards.is_empty())
                .unwrap_or(false)
            {
                app.start_learn_mode(stats);
            }
        }
        KeyCode::Char('r') => {
            if app.current_deck().is_some() {
                app.mode = AppMode::SpacedRep;
                app.card_flipped = false;
            }
        }
        KeyCode::Char('s') => {
            if app.current_deck().is_some() {
                app.shuffle = true;
                shuffle_deck(app);
            }
        }
        KeyCode::Char('S') => {
            app.shuffle = false;
            app.shuffled_order.clear();
            app.selected_card = 0;
        }
        KeyCode::Char('e') => {
            if let Some(deck) = app.current_deck() {
                let path = deck.file_path.clone();
                open_in_editor(app, &path);
            }
        }
        KeyCode::Char('a') => app.open_add_card(),
        KeyCode::Char('d') => app.delete_current_card(),
        KeyCode::Char('t') => app.mode = AppMode::ThemePicker,
        KeyCode::Char('T') => {
            let next = app.theme.name.next();
            app.set_theme(next);
            let _ = config.set_theme(next);
            app.set_status(format!("theme: {}", next.display_name()));
        }
        _ => {}
    }
}

fn handle_search(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.mode = AppMode::Browse;
            app.search_buffer.clear();
        }
        KeyCode::Backspace => {
            if app.search_buffer.is_empty() {
                app.mode = AppMode::Browse;
            } else {
                app.search_buffer.pop();
            }
        }
        KeyCode::Enter => {
            app.execute_search();
            app.mode = AppMode::Browse;
        }
        KeyCode::Char(c) => {
            app.search_buffer.push(c);
        }
        _ => {}
    }
}

fn handle_flashcard(
    app: &mut App,
    key: KeyEvent,
    stats: &mut StatsStore,
    config: &mut ConfigStore,
) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.mode = AppMode::Browse;
            app.card_flipped = false;
            app.back_scroll = 0;
            config.config.deck_positions = app.deck_positions.clone();
            let _ = stats.save();
            let _ = config.save();
        }
        KeyCode::Char(':') => {
            app.mode = AppMode::Command;
            app.command_buffer = ":".to_string();
            app.command_cursor = 1;
        }
        KeyCode::Tab | KeyCode::Right => {
            app.focused_pane = cycle_pane_next(app.focused_pane);
        }
        KeyCode::BackTab | KeyCode::Left => {
            app.focused_pane = cycle_pane_prev(app.focused_pane);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.card_flipped {
                app.scroll_back_up();
            } else {
                app.prev_card();
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.card_flipped {
                app.scroll_back_down();
            } else {
                app.next_card();
            }
        }
        KeyCode::Char(' ') => app.flip_card(),
        KeyCode::Char('s') => {
            app.shuffle = !app.shuffle;
            if app.shuffle {
                shuffle_deck(app);
            }
        }
        KeyCode::Char('1') if app.card_flipped => rate_card(app, stats, 1),
        KeyCode::Char('2') if app.card_flipped => rate_card(app, stats, 3),
        KeyCode::Char('3') if app.card_flipped => rate_card(app, stats, 4),
        KeyCode::Char('4') if app.card_flipped => rate_card(app, stats, 5),
        _ => {}
    }
}

fn handle_learn(app: &mut App, key: KeyEvent, stats: &mut StatsStore) {
    if app.learn_awaiting_next {
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Esc => {
                app.learn_awaiting_next = false;
                app.input_clear();
                app.learn_last_score = None;
                app.learn_last_answer = None;
                app.learn_last_expected = None;
                if let Some(&next_idx) = app.learn_remaining.first() {
                    app.selected_card = next_idx;
                    app.save_current_position();
                }
            }
            KeyCode::Char('q') => {
                app.mode = AppMode::Browse;
                app.learn_awaiting_next = false;
                app.input_clear();
            }
            KeyCode::Char(':') => {
                app.mode = AppMode::Command;
                app.command_buffer = ":".to_string();
                app.command_cursor = 1;
            }
            _ => {}
        }
        return;
    }

    if app.learn_remaining.is_empty() {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter | KeyCode::Char(' ') => {
                app.mode = AppMode::Browse;
                app.input_clear();
            }
            KeyCode::Char(':') => {
                app.mode = AppMode::Command;
                app.command_buffer = ":".to_string();
                app.command_cursor = 1;
            }
            _ => {}
        }
        return;
    }
    let card_idx = match app.learn_remaining.first() {
        Some(&idx) => idx,
        None => return,
    };

    let (card_id, card_back) = if let Some(deck) = app.current_deck() {
        deck.cards
            .get(card_idx)
            .map(|c| (c.id.clone(), c.back.clone()))
            .unwrap_or_default()
    } else {
        return;
    };

    let is_multiline = card_back.contains('\n');
    let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let is_alt = key.modifiers.contains(KeyModifiers::ALT);
    let is_submit = (is_ctrl && key.code == KeyCode::Enter)
        || (is_ctrl && key.code == KeyCode::Char('s'))
        || (is_alt && key.code == KeyCode::Enter)
        || (app.input_edit_mode == InputEditMode::Normal && key.code == KeyCode::Enter)
        || (!is_multiline
            && key.code == KeyCode::Enter
            && app.input_edit_mode == InputEditMode::Insert);

    if is_submit {
        let answer = app.input_buffer.clone();
        let score = matcher::score(&answer, &card_back);
        let epoch = current_epoch();
        if matcher::is_match(&answer, &card_back) {
            app.learn_remaining.remove(0);
            let mut state = stats.get_or_default(&card_id);
            sm2::update_sm2(&mut state, 4, epoch);
            let _ = stats.update(state);
        } else {
            let idx = app.learn_remaining.remove(0);
            app.learn_remaining.push(idx);
            let mut state = stats.get_or_default(&card_id);
            sm2::update_sm2(&mut state, 1, epoch);
            let _ = stats.update(state);
        }
        app.learn_last_score = Some(score);
        app.learn_last_answer = Some(answer);
        app.learn_last_expected = Some(card_back);
        app.learn_awaiting_next = true;
        return;
    }

    match app.input_edit_mode {
        InputEditMode::Normal => match key.code {
            KeyCode::Esc => {
                app.mode = AppMode::Browse;
                app.learn_last_score = None;
            }
            KeyCode::Char('i') => app.input_edit_mode = InputEditMode::Insert,
            KeyCode::Char('a') => {
                app.move_cursor_right();
                app.input_edit_mode = InputEditMode::Insert;
            }
            KeyCode::Char('o') => {
                app.input_push('\n');
                app.input_edit_mode = InputEditMode::Insert;
            }
            KeyCode::Char('h') | KeyCode::Left => app.move_cursor_left(),
            KeyCode::Char('l') | KeyCode::Right => app.move_cursor_right(),
            KeyCode::Char('k') | KeyCode::Up => app.move_cursor_up(),
            KeyCode::Char('j') | KeyCode::Down => app.move_cursor_down(),
            KeyCode::Char('0') => app.move_line_start(),
            KeyCode::Char('$') => app.move_line_end(),
            KeyCode::Char('g') => {
                app.input_cursor = 0;
                app.input_preferred_col = 0;
            }
            KeyCode::Char('G') => {
                app.input_cursor = app.input_buffer.len();
            }
            KeyCode::Char('w') | KeyCode::Char('b') => app.delete_word_before_cursor(),
            KeyCode::Char('x') => app.delete_char_at_cursor(),
            KeyCode::Char('d') => app.delete_line(),
            KeyCode::Char(';') => {
                app.mode = AppMode::Command;
                app.command_buffer = ":".to_string();
                app.command_cursor = 1;
            }
            KeyCode::Tab => {
                app.focused_pane = cycle_pane_next(app.focused_pane);
            }
            KeyCode::BackTab => {
                app.focused_pane = cycle_pane_prev(app.focused_pane);
            }
            _ => {}
        },
        InputEditMode::Insert => {
            if (is_ctrl && key.code == KeyCode::Char('w'))
                || (is_alt && key.code == KeyCode::Backspace)
                || (is_ctrl && key.code == KeyCode::Backspace)
            {
                app.delete_word_before_cursor();
                return;
            }

            match key.code {
                KeyCode::Esc => app.input_edit_mode = InputEditMode::Normal,
                KeyCode::Up => app.move_cursor_up(),
                KeyCode::Down => app.move_cursor_down(),
                KeyCode::Left => app.move_cursor_left(),
                KeyCode::Right => app.move_cursor_right(),
                KeyCode::Enter if is_multiline => app.input_push('\n'),
                KeyCode::Backspace => app.input_backspace(),
                KeyCode::Char(c) => app.input_push(c),
                KeyCode::Tab => {
                    app.focused_pane = cycle_pane_next(app.focused_pane);
                }
                KeyCode::BackTab => {
                    app.focused_pane = cycle_pane_prev(app.focused_pane);
                }
                _ => {}
            }
        }
    }
}

fn handle_spaced_rep(
    app: &mut App,
    key: KeyEvent,
    stats: &mut StatsStore,
    config: &mut ConfigStore,
) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.mode = AppMode::Browse;
            app.card_flipped = false;
            app.back_scroll = 0;
            config.config.deck_positions = app.deck_positions.clone();
            let _ = stats.save();
            let _ = config.save();
        }
        KeyCode::Char(':') => {
            app.mode = AppMode::Command;
            app.command_buffer = ":".to_string();
            app.command_cursor = 1;
        }
        KeyCode::Tab | KeyCode::Right => {
            app.focused_pane = cycle_pane_next(app.focused_pane);
        }
        KeyCode::BackTab | KeyCode::Left => {
            app.focused_pane = cycle_pane_prev(app.focused_pane);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.card_flipped {
                app.scroll_back_up();
            } else {
                app.prev_card();
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.card_flipped {
                app.scroll_back_down();
            } else {
                app.next_card();
            }
        }
        KeyCode::Char(' ') => app.flip_card(),
        KeyCode::Char('1') if app.card_flipped => rate_current_due(app, stats, 1),
        KeyCode::Char('2') if app.card_flipped => rate_current_due(app, stats, 3),
        KeyCode::Char('3') if app.card_flipped => rate_current_due(app, stats, 4),
        KeyCode::Char('4') if app.card_flipped => rate_current_due(app, stats, 5),
        _ => {}
    }
}

fn handle_theme_picker(app: &mut App, key: KeyEvent, config: &mut ConfigStore) {
    match key.code {
        KeyCode::Esc => app.mode = AppMode::Browse,
        KeyCode::Up | KeyCode::Char('k') => {
            let count = ThemeName::ALL.len();
            app.theme_picker_selected = (app.theme_picker_selected + count - 1) % count;
            let selected = ThemeName::ALL[app.theme_picker_selected];
            app.set_theme(selected);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.theme_picker_selected = (app.theme_picker_selected + 1) % ThemeName::ALL.len();
            let selected = ThemeName::ALL[app.theme_picker_selected];
            app.set_theme(selected);
        }
        KeyCode::Enter => {
            let selected = ThemeName::ALL[app.theme_picker_selected];
            app.set_theme(selected);
            let _ = config.set_theme(selected);
            app.mode = AppMode::Browse;
            app.set_status(format!("theme: {}", selected.display_name()));
        }
        KeyCode::Char('T') => {
            app.theme_picker_selected = (app.theme_picker_selected + 1) % ThemeName::ALL.len();
            let selected = ThemeName::ALL[app.theme_picker_selected];
            app.set_theme(selected);
        }
        _ => {}
    }
}

fn handle_command(app: &mut App, key: KeyEvent, stats: &mut StatsStore, config: &mut ConfigStore) {
    match key.code {
        KeyCode::Esc => {
            app.mode = AppMode::Browse;
            app.command_buffer.clear();
        }
        KeyCode::Backspace => {
            if app.command_buffer.len() <= 1 {
                app.mode = AppMode::Browse;
                app.command_buffer.clear();
            } else {
                app.command_buffer.pop();
            }
        }
        KeyCode::Enter => {
            execute_command(app, stats, config);
        }
        KeyCode::Char(c) => {
            app.command_buffer.push(c);
        }
        _ => {}
    }
}

fn execute_command(app: &mut App, stats: &mut StatsStore, config: &mut ConfigStore) {
    let raw = app.command_buffer.trim_start_matches(':').trim();
    let parts: Vec<&str> = raw.split_whitespace().collect();
    if parts.is_empty() {
        app.mode = AppMode::Browse;
        app.command_buffer.clear();
        return;
    }

    let cmd = parts[0];
    let arg = parts.get(1).copied().unwrap_or("");
    match cmd {
        "q" | "quit" => app.should_quit = true,
        "w" | "write" => {
            config.config.deck_positions = app.deck_positions.clone();
            let res_s = stats.save();
            let res_c = config.save();
            if res_s.is_ok() && res_c.is_ok() {
                app.set_status("stats & config saved");
            } else {
                app.set_status("error saving data");
            }
        }
        "wq" | "x" => {
            config.config.deck_positions = app.deck_positions.clone();
            let _ = stats.save();
            let _ = config.save();
            app.should_quit = true;
        }
        "set" | "theme" => {
            let theme_str = if cmd == "set" {
                arg.trim_start_matches("theme=")
                    .trim_start_matches("theme:")
            } else {
                arg
            };
            if let Some(t_name) = ThemeName::from_cli_str(theme_str) {
                app.set_theme(t_name);
                let _ = config.set_theme(t_name);
                app.set_status(format!("theme set to {}", t_name.display_name()));
            } else {
                app.set_status(format!("unknown theme: {}", theme_str));
            }
        }
        "s" | "shuffle" => {
            if app.current_deck().is_some() {
                app.shuffle = true;
                shuffle_deck(app);
                app.set_status("deck shuffled");
            }
        }
        "S" | "unshuffle" => {
            app.shuffle = false;
            app.shuffled_order.clear();
            app.set_status("original deck order restored");
        }
        "l" | "learn" => {
            if app
                .current_deck()
                .map(|d| !d.cards.is_empty())
                .unwrap_or(false)
            {
                app.start_learn_mode(stats);
            }
        }
        "fc" | "flashcard" | "flashcards" => {
            if app.current_deck().is_some() {
                app.mode = AppMode::Flashcard;
                app.card_flipped = false;
            }
        }
        "r" | "review" => {
            if app.current_deck().is_some() {
                app.mode = AppMode::SpacedRep;
                app.card_flipped = false;
            }
        }
        "e" | "edit" => {
            if let Some(deck) = app.current_deck() {
                let path = deck.file_path.clone();
                open_in_editor(app, &path);
            }
        }
        "a" | "add" => app.open_add_card(),
        "d" | "del" | "delete" => app.delete_current_card(),
        "reset" => {
            if let Some(deck) = app.current_deck() {
                let ids: Vec<String> = deck.cards.iter().map(|c| c.id.clone()).collect();
                for id in &ids {
                    stats.states.remove(id);
                }
                match stats.save() {
                    Ok(_) => app.set_status(format!("reset progress for {} card(s)", ids.len())),
                    Err(e) => app.set_status(format!("reset failed: {}", e)),
                }
            } else {
                app.set_status("no deck selected");
            }
        }
        "h" | "help" | "?" => {
            app.set_status(
                ":w | :q | :wq | :a (add) | :d (delete) | :reset | :set theme=<name> | :s (shuffle) | :l | :fc | :r",
                );
        }
        _ => {
            app.set_status(format!("unknown command: :{}\nuse :help", cmd));
        }
    }
    app.mode = AppMode::Browse;
    app.command_buffer.clear();
}

fn handle_add_card(app: &mut App, key: KeyEvent) {
    let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let is_alt = key.modifiers.contains(KeyModifiers::ALT);
    if (is_ctrl && key.code == KeyCode::Char('s'))
        || (is_ctrl && key.code == KeyCode::Enter)
        || (is_alt && key.code == KeyCode::Enter)
    {
        save_new_card(app);
        return;
    }

    match key.code {
        KeyCode::Esc => {
            app.add_card_front.clear();
            app.add_card_back.clear();
            app.mode = AppMode::Browse;
        }
        KeyCode::Tab | KeyCode::Down | KeyCode::Up => {
            app.add_card_focus = match app.add_card_focus {
                AddCardFocus::Front => AddCardFocus::Back,
                AddCardFocus::Back => AddCardFocus::Front,
            };
        }
        KeyCode::Enter => {
            if app.add_card_focus == AddCardFocus::Back {
                save_new_card(app);
            } else {
                app.add_card_focus = AddCardFocus::Back;
            }
        }
        KeyCode::Backspace => match app.add_card_focus {
            AddCardFocus::Front => {
                app.add_card_front.pop();
            }
            AddCardFocus::Back => {
                app.add_card_back.pop();
            }
        },
        KeyCode::Char(c) => match app.add_card_focus {
            AddCardFocus::Front => app.add_card_front.push(c),
            AddCardFocus::Back => app.add_card_back.push(c),
        },
        _ => {}
    }
}

fn save_new_card(app: &mut App) {
    let front = app.add_card_front.trim().to_string();
    let back = app.add_card_back.trim().to_string();
    if front.is_empty() || back.is_empty() {
        app.set_status("both front and back fields must not be empty");
        return;
    }
    if let Some(deck) = app.decks.get_mut(app.selected_deck) {
        let mut next_idx = deck.cards.len();
        while deck
            .cards
            .iter()
            .any(|c| c.id == format!("{}-{}", deck.id, next_idx))
        {
            next_idx += 1;
        }
        let new_card = crate::models::card::Card {
            id: format!("{}-{}", deck.id, next_idx),
            front,
            back,
            hints: vec![],
            tags: vec![],
        };
        let new_card_real_idx = deck.cards.len();
        deck.cards.push(new_card);
        let _ = crate::storage::deck_loader::save_deck(deck);
        if !app.shuffled_order.is_empty() {
            app.shuffled_order.push(new_card_real_idx);
        }
        app.selected_card = deck.cards.len() - 1;
        app.save_current_position();
        app.add_card_front.clear();
        app.add_card_back.clear();
        app.mode = AppMode::Browse;
        app.set_status("card added and saved to disk");
    }
}

fn rate_card(app: &mut App, stats: &mut StatsStore, quality: u8) {
    if let Some(card_idx) = app.current_card_index() {
        let card_id = app
            .current_deck()
            .and_then(|d| d.cards.get(card_idx))
            .map(|c| c.id.clone());
        if let Some(id) = card_id {
            let mut state = stats.get_or_default(&id);
            sm2::update_sm2(&mut state, quality, current_epoch());
            let _ = stats.update(state);
        }
    }
    app.next_card();
    app.card_flipped = false;
}

fn rate_current_due(app: &mut App, stats: &mut StatsStore, quality: u8) {
    let epoch = current_epoch();
    let due_ids = stats.due_ids(epoch);
    let card_id = app.current_deck().and_then(|deck| {
        deck.cards
            .iter()
            .find(|c| due_ids.contains(&c.id))
            .map(|c| c.id.clone())
    });
    if let Some(id) = card_id {
        let mut state = stats.get_or_default(&id);
        sm2::update_sm2(&mut state, quality, epoch);
        let _ = stats.update(state);
    }
    app.card_flipped = false;
    app.back_scroll = 0;
}

fn shuffle_deck(app: &mut App) {
    if let Some(deck) = app.current_deck() {
        let n = deck.cards.len();
        let mut indices: Vec<usize> = (0..n).collect();
        let seed = current_epoch() as u64;
        let mut rng = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        for i in (1..n).rev() {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let j = (rng >> 33) as usize % (i + 1);
            indices.swap(i, j);
        }
        app.shuffled_order = indices;
        app.selected_card = 0;
    }
}

fn open_in_editor(app: &mut App, path: &std::path::Path) {
    use crossterm::execute;
    use crossterm::terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
    };
    use std::io::stdout;

    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "nvim".to_string());
    let mut out = stdout();
    let _ = disable_raw_mode();
    let _ = execute!(out, LeaveAlternateScreen);
    let path_str = path.to_string_lossy();
    let shell_cmd = format!("{} {}", editor, path_str);
    let _ = std::process::Command::new("sh")
        .arg("-c")
        .arg(&shell_cmd)
        .status();
    let _ = enable_raw_mode();
    let _ = execute!(out, EnterAlternateScreen);

    if let Ok(updated_deck) = crate::storage::deck_loader::load_deck(path) {
        if let Some(deck) = app.decks.get_mut(app.selected_deck) {
            *deck = updated_deck;
            app.set_status("deck reloaded from disk");
        }
    }
    app.needs_clear = true;
}

pub fn current_epoch() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::theme::Theme;

    #[test]
    fn test_handle_learn_empty_remaining_exit() {
        let mut app = App::new(vec![], Theme::default());
        let mut stats = StatsStore {
            states: std::collections::HashMap::new(),
            path: std::path::PathBuf::from("/tmp/test_stats.json"),
        };
        app.mode = AppMode::Learn;
        app.learn_remaining.clear();
        app.learn_awaiting_next = false;
        handle_learn(&mut app, KeyEvent::from(KeyCode::Char('q')), &mut stats);
        assert_eq!(app.mode, AppMode::Browse);

        app.mode = AppMode::Learn;
        handle_learn(&mut app, KeyEvent::from(KeyCode::Esc), &mut stats);
        assert_eq!(app.mode, AppMode::Browse);

        app.mode = AppMode::Learn;
        handle_learn(&mut app, KeyEvent::from(KeyCode::Enter), &mut stats);
        assert_eq!(app.mode, AppMode::Browse);

        app.mode = AppMode::Learn;
        handle_learn(&mut app, KeyEvent::from(KeyCode::Char(':')), &mut stats);
        assert_eq!(app.mode, AppMode::Command);
    }

    #[test]
    fn test_vim_keymaps_and_search() {
        use crate::models::card::{Card, Deck};
        let cards = vec![
            Card {
                id: "1".into(),
                front: "Aplle".into(),
                back: "Fruit".into(),
                hints: vec![],
                tags: vec![],
            },
            Card {
                id: "2".into(),
                front: "Banana".into(),
                back: "Yellow".into(),
                hints: vec![],
                tags: vec![],
            },
            Card {
                id: "3".into(),
                front: "Cherry".into(),
                back: "Red fruit".into(),
                hints: vec![],
                tags: vec![],
            },
        ];
        let deck = Deck {
            id: "d1".into(),
            title: "test deck".into(),
            description: Some("".into()),
            file_path: std::path::PathBuf::from("test.md"),
            format: crate::models::card::DeckFormat::Markdown,
            cards,
        };
        let mut app = App::new(vec![deck], Theme::default());
        let mut stats = StatsStore {
            states: std::collections::HashMap::new(),
            path: std::path::PathBuf::from("/tmp/test_stats.json"),
        };
        let mut config = ConfigStore {
            config: Default::default(),
            path: std::path::PathBuf::from("/tmp/test_cfg.json"),
        };

        app.focused_pane = FocusedPane::CardList;
        app.selected_card = 2;
        handle_browse(
            &mut app,
            KeyEvent::from(KeyCode::Char('g')),
            &mut stats,
            &mut config,
        );
        assert!(app.pending_g);
        handle_browse(
            &mut app,
            KeyEvent::from(KeyCode::Char('g')),
            &mut stats,
            &mut config,
        );
        assert_eq!(app.selected_card, 0);

        handle_browse(
            &mut app,
            KeyEvent::from(KeyCode::Char('G')),
            &mut stats,
            &mut config,
        );
        assert_eq!(app.selected_card, 2);

        handle_browse(
            &mut app,
            KeyEvent::from(KeyCode::Char('/')),
            &mut stats,
            &mut config,
        );
        assert_eq!(app.mode, AppMode::Search);

        for c in "fruit".chars() {
            handle_search(&mut app, KeyEvent::from(KeyCode::Char(c)));
        }
        assert_eq!(app.search_buffer, "fruit");

        handle_search(&mut app, KeyEvent::from(KeyCode::Enter));
        assert_eq!(app.mode, AppMode::Browse);
        assert_eq!(app.search_results.len(), 3);
        assert_eq!(app.selected_card, 0);

        handle_browse(
            &mut app,
            KeyEvent::from(KeyCode::Char('n')),
            &mut stats,
            &mut config,
        );
        assert_eq!(app.selected_card, 1);

        handle_browse(
            &mut app,
            KeyEvent::from(KeyCode::Char('n')),
            &mut stats,
            &mut config,
        );
        assert_eq!(app.selected_card, 2);

        handle_browse(
            &mut app,
            KeyEvent::from(KeyCode::Char('N')),
            &mut stats,
            &mut config,
        );
        assert_eq!(app.selected_card, 1);

        handle_browse(
            &mut app,
            KeyEvent::from(KeyCode::Esc),
            &mut stats,
            &mut config,
        );
        assert!(!app.search_query.is_empty());
        assert_eq!(app.search_esc_count, 1);

        handle_browse(
            &mut app,
            KeyEvent::from(KeyCode::Esc),
            &mut stats,
            &mut config,
        );
        assert!(!app.search_query.is_empty());
        assert!(!app.search_results.is_empty());
        assert_eq!(app.search_esc_count, 0);
    }
}
