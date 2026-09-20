use crate::models::card::Deck;
use crate::models::theme::{Theme, ThemeName};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusedPane {
    #[default]
    DeckList,
    CardList,
    MainView,
    Stats,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputEditMode {
    #[default]
    Insert,
    Normal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AddCardFocus {
    #[default]
    Front,
    Back,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppMode {
    #[default]
    Browse,
    Flashcard,
    Learn,
    SpacedRep,
    ThemePicker,
    Command,
    AddCard,
    Search,
}

pub struct App {
    pub decks: Vec<Deck>,
    pub selected_deck: usize,
    pub selected_card: usize,

    pub mode: AppMode,
    pub focused_pane: FocusedPane,

    pub theme: Theme,
    pub theme_picker_selected: usize,

    pub input_buffer: String,
    pub input_cursor: usize,
    pub input_edit_mode: InputEditMode,
    pub input_preferred_col: usize,
    pub command_buffer: String,
    pub command_cursor: usize,

    pub add_card_front: String,
    pub add_card_back: String,
    pub add_card_focus: AddCardFocus,

    pub deck_positions: HashMap<String, usize>,
    pub status_message: Option<String>,
    pub shuffle: bool,
    pub shuffled_order: Vec<usize>,
    pub card_flipped: bool,
    pub learn_remaining: Vec<usize>,
    pub learn_last_score: Option<f64>,
    pub learn_awaiting_next: bool,
    pub learn_last_answer: Option<String>,
    pub learn_last_expected: Option<String>,

    pub needs_clear: bool,
    pub should_quit: bool,
    pub card_list_scroll: usize,
    pub back_scroll: u16,
    pub search_buffer: String,
    pub search_query: String,
    pub search_results: Vec<usize>,
    pub search_result_idx: usize,
    pub search_esc_count: u8,
    pub pending_g: bool,
}

impl App {
    pub fn new(decks: Vec<Deck>, theme: Theme) -> Self {
        Self {
            decks,
            selected_deck: 0,
            selected_card: 0,
            mode: AppMode::default(),
            focused_pane: FocusedPane::default(),
            theme_picker_selected: ThemeName::ALL
                .iter()
                .position(|&t| t == theme.name)
                .unwrap_or(0),
            theme,
            input_buffer: String::new(),
            input_cursor: 0,
            input_edit_mode: InputEditMode::Insert,
            input_preferred_col: 0,
            command_buffer: String::new(),
            command_cursor: 0,
            add_card_front: String::new(),
            add_card_back: String::new(),
            add_card_focus: AddCardFocus::default(),
            deck_positions: HashMap::new(),
            status_message: None,
            shuffle: false,
            shuffled_order: Vec::new(),
            card_flipped: false,
            learn_remaining: Vec::new(),
            learn_last_score: None,
            learn_awaiting_next: false,
            learn_last_answer: None,
            learn_last_expected: None,
            needs_clear: false,
            should_quit: false,
            card_list_scroll: 0,
            back_scroll: 0,
            search_buffer: String::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            search_result_idx: 0,
            search_esc_count: 0,
            pending_g: false,
        }
    }

    pub fn current_deck(&self) -> Option<&Deck> {
        self.decks.get(self.selected_deck)
    }

    pub fn current_card_index(&self) -> Option<usize> {
        let deck = self.current_deck()?;
        if deck.cards.is_empty() {
            return None;
        }
        let raw_idx = if self.shuffle && !self.shuffled_order.is_empty() {
            let s_idx = self
                .selected_card
                .min(self.shuffled_order.len().saturating_sub(1));
            self.shuffled_order.get(s_idx).copied().unwrap_or(0)
        } else {
            self.selected_card
        };
        Some(raw_idx.min(deck.cards.len().saturating_sub(1)))
    }

    pub fn deck_key(deck: &Deck) -> String {
        let path = &deck.file_path;
        let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let path_str = canonical.to_string_lossy().to_string();
        if !path_str.is_empty() {
            path_str
        } else {
            deck.id.clone()
        }
    }

    pub fn save_current_position(&mut self) {
        if let Some(deck) = self.current_deck() {
            let key = Self::deck_key(deck);
            self.deck_positions.insert(key, self.selected_card);
        }
    }

    pub fn select_deck(&mut self, new_idx: usize) {
        if self.decks.is_empty() {
            return;
        }
        self.save_current_position();

        let clamped = new_idx.min(self.decks.len() - 1);
        self.selected_deck = clamped;
        self.card_flipped = false;
        self.shuffle = false;
        self.shuffled_order.clear();
        self.card_list_scroll = 0;
        if let Some(new_deck) = self.current_deck() {
            let key = Self::deck_key(new_deck);
            let saved = self.deck_positions.get(&key).copied().unwrap_or(0);
            self.selected_card = saved.min(new_deck.cards.len().saturating_sub(1));
        }
    }

    pub fn prev_deck(&mut self) {
        let target = self.selected_deck.saturating_sub(1);
        self.select_deck(target);
    }

    pub fn next_deck(&mut self) {
        let target = self.selected_deck + 1;
        self.select_deck(target);
    }

    pub fn prev_card(&mut self) {
        self.selected_card = self.selected_card.saturating_sub(1);
        self.card_flipped = false;
        self.save_current_position();
    }

    pub fn next_card(&mut self) {
        if let Some(deck) = self.current_deck() {
            let max = deck.cards.len().saturating_sub(1);
            self.selected_card = (self.selected_card + 1).min(max);
            self.card_flipped = false;
        }
        self.save_current_position();
    }

    pub fn jump_to_first(&mut self) {
        match self.focused_pane {
            FocusedPane::DeckList => self.select_deck(0),
            FocusedPane::CardList | FocusedPane::MainView | FocusedPane::Stats => {
                self.selected_card = 0;
                self.card_flipped = false;
                self.save_current_position();
            }
        }
    }

    pub fn jump_to_last(&mut self) {
        match self.focused_pane {
            FocusedPane::DeckList => {
                let max = self.decks.len().saturating_sub(1);
                self.select_deck(max);
            }
            FocusedPane::CardList | FocusedPane::MainView | FocusedPane::Stats => {
                if let Some(deck) = self.current_deck() {
                    let max = deck.cards.len().saturating_sub(1);
                    self.selected_card = max;
                    self.card_flipped = false;
                    self.save_current_position();
                }
            }
        }
    }

    pub fn jump_to_middle(&mut self) {
        match self.focused_pane {
            FocusedPane::DeckList => {
                let mid = self.decks.len() / 2;
                self.select_deck(mid);
            }
            FocusedPane::CardList | FocusedPane::MainView | FocusedPane::Stats => {
                if let Some(deck) = self.current_deck() {
                    let mid = deck.cards.len() / 2;
                    self.selected_card = mid;
                    self.card_flipped = false;
                    self.save_current_position();
                }
            }
        }
    }

    pub fn scroll_half_page_down(&mut self) {
        match self.focused_pane {
            FocusedPane::DeckList => {
                let target = self.selected_deck + 5;
                self.select_deck(target);
            }
            FocusedPane::CardList | FocusedPane::MainView | FocusedPane::Stats => {
                if let Some(deck) = self.current_deck() {
                    let max = deck.cards.len().saturating_sub(1);
                    self.selected_card = (self.selected_card + 5).min(max);
                    self.card_flipped = false;
                    self.save_current_position();
                }
            }
        }
    }

    pub fn scroll_half_page_up(&mut self) {
        match self.focused_pane {
            FocusedPane::DeckList => {
                let target = self.selected_deck.saturating_sub(5);
                self.select_deck(target);
            }
            FocusedPane::CardList | FocusedPane::MainView | FocusedPane::Stats => {
                self.selected_card = self.selected_card.saturating_sub(5);
                self.card_flipped = false;
                self.save_current_position();
            }
        }
    }

    pub fn execute_search(&mut self) {
        let query = self.search_buffer.trim().to_string();
        self.search_query = query.clone();
        self.search_results.clear();
        self.search_result_idx = 0;
        self.search_esc_count = 0;

        if query.is_empty() {
            self.set_status("search query cleared");
            return;
        }

        let q_lower = query.to_lowercase();
        let mut matches = Vec::new();
        if let Some(deck) = self.current_deck() {
            let card_count = deck.cards.len();
            for pos in 0..card_count {
                let real_idx = if self.shuffle && !self.shuffled_order.is_empty() {
                    self.shuffled_order.get(pos).copied().unwrap_or(pos)
                } else {
                    pos
                };
                if let Some(card) = deck.cards.get(real_idx) {
                    if card.front.to_lowercase().contains(&q_lower)
                        || card.back.to_lowercase().contains(&q_lower)
                        || card
                            .hints
                            .iter()
                            .any(|h| h.to_lowercase().contains(&q_lower))
                        || card
                            .tags
                            .iter()
                            .any(|t| t.to_lowercase().contains(&q_lower))
                    {
                        matches.push(pos);
                    }
                }
            }
        }
        self.search_results = matches;
        if self.search_results.is_empty() {
            self.set_status(format!("no matches found for '{}'", query));
        } else {
            self.search_result_idx = 0;
            self.selected_card = self.search_results[0];
            self.save_current_position();
            self.set_status(format!(
                "1/{} matches for '{}'",
                self.search_results.len(),
                query
            ));
        }
    }

    pub fn next_search_match(&mut self) {
        if self.search_results.is_empty() {
            if !self.search_query.is_empty() {
                self.set_status(format!("no matches for '{}'", self.search_query));
            } else {
                self.set_status("no active search. press '/' to search");
            }
            return;
        }
        self.search_result_idx = (self.search_result_idx + 1) % self.search_results.len();
        self.selected_card = self.search_results[self.search_result_idx];
        self.save_current_position();
        self.set_status(format!(
            "{}/{} matches for ''{}'",
            self.search_result_idx + 1,
            self.search_results.len(),
            self.search_query
        ));
    }

    pub fn prev_search_match(&mut self) {
        if self.search_results.is_empty() {
            if !self.search_query.is_empty() {
                self.set_status(format!("no matches for '{}'", self.search_query));
            } else {
                self.set_status("no active search");
            }
            return;
        }
        self.search_result_idx =
            (self.search_result_idx + self.search_results.len() - 1) % self.search_results.len();
        self.selected_card = self.search_results[self.search_result_idx];
        self.save_current_position();
        self.set_status(format!(
            "{}/{} matches for '{}'",
            self.search_result_idx + 1,
            self.search_results.len(),
            self.search_query
        ));
    }

    pub fn clear_search(&mut self) {
        self.search_buffer.clear();
        self.search_query.clear();
        self.search_results.clear();
        self.search_esc_count = 0;
        self.set_status("search cleared")
    }

    pub fn flip_card(&mut self) {
        self.card_flipped = !self.card_flipped;
        self.back_scroll = 0;
    }

    pub fn scroll_back_down(&mut self) {
        self.back_scroll = self.back_scroll.saturating_add(1);
    }

    pub fn scroll_back_up(&mut self) {
        self.back_scroll = self.back_scroll.saturating_sub(1);
    }

    pub fn set_theme(&mut self, name: ThemeName) {
        self.theme = Theme::from_name(name);
        self.theme_picker_selected = ThemeName::ALL.iter().position(|&t| t == name).unwrap_or(0);
    }

    #[allow(dead_code)]
    pub fn cycle_theme(&mut self) {
        self.set_theme(self.theme.name.next());
    }
    pub fn start_learn_mode(&mut self, stats: &crate::storage::stats_store::StatsStore) {
        let mut card_count = 0;
        let mut unmastered = Vec::new();

        if let Some(deck) = self.current_deck() {
            card_count = deck.cards.len();
            unmastered = deck
                .cards
                .iter()
                .enumerate()
                .filter(|(_, card)| stats.get_or_default(&card.id).repetitions == 0)
                .map(|(i, _)| i)
                .collect();
        }

        if card_count > 0 {
            if unmastered.is_empty() {
                self.learn_remaining = (0..card_count).collect();
            } else {
                self.learn_remaining = unmastered;
            }

            let target_idx = self
                .current_card_index()
                .unwrap_or(0)
                .min(card_count.saturating_sub(1));
            if let Some(pos) = self
                .learn_remaining
                .iter()
                .position(|&idx| idx == target_idx)
            {
                let active = self.learn_remaining.remove(pos);
                self.learn_remaining.insert(0, active);
            } else {
                self.learn_remaining.insert(0, target_idx);
            }
        }

        self.input_clear();
        self.learn_last_score = None;
        self.learn_awaiting_next = false;
        self.learn_last_answer = None;
        self.learn_last_expected = None;
        self.card_flipped = false;
        self.mode = AppMode::Learn;
        if let Some(&first_idx) = self.learn_remaining.first() {
            self.selected_card = first_idx;
            self.save_current_position();
        }
    }

    pub fn open_add_card(&mut self) {
        if self.current_deck().is_some() {
            self.add_card_front.clear();
            self.add_card_back.clear();
            self.add_card_focus = AddCardFocus::Front;
            self.mode = AppMode::AddCard;
        } else {
            self.set_status("no deck selected");
        }
    }

    pub fn delete_current_card(&mut self) {
        let card_idx = self.current_card_index().unwrap_or(self.selected_card);
        let selected_deck_idx = self.selected_deck;
        if let Some(deck) = self.decks.get_mut(selected_deck_idx) {
            if deck.cards.is_empty() {
                self.set_status("no cards to delete");
                return;
            }
            if card_idx < deck.cards.len() {
                let removed = deck.cards.remove(card_idx);
                let _ = crate::storage::deck_loader::save_deck(deck);
                let snippet: String = removed.front.chars().take(20).collect();

                self.shuffled_order.retain(|&x| x != card_idx);
                for idx in self.shuffled_order.iter_mut() {
                    if *idx > card_idx {
                        *idx -= 1;
                    }
                }

                let new_len = deck.cards.len();
                if self.shuffle && !self.shuffled_order.is_empty() {
                    self.selected_card = self.selected_card.min(new_len.saturating_sub(1));
                }
                self.save_current_position();
                self.set_status(format!("deleted card: '{}'", snippet));
            }
        }
    }

    pub fn cursor_line_col(&self) -> (usize, usize) {
        let text = &self.input_buffer;
        let cursor_pos = self.input_cursor.min(text.len());
        let mut line_idx = 0;
        let mut col_idx = 0;
        for (i, c) in text.char_indices() {
            if i >= cursor_pos {
                break;
            }
            if c == '\n' {
                line_idx += 1;
                col_idx = 0;
            } else {
                col_idx += 1;
            }
        }
        (line_idx, col_idx)
    }

    pub fn byte_offset_for_line_col(&self, target_line: usize, target_col: usize) -> usize {
        let text = &self.input_buffer;
        let lines: Vec<&str> = text.split('\n').collect();
        if lines.is_empty() {
            return 0;
        }
        let line_i = target_line.min(lines.len() - 1);
        let line_str = lines[line_i];
        let char_count = line_str.chars().count();
        let col_i = target_col.min(char_count);
        let mut offset = 0;

        for line_str in lines.iter().take(line_i) {
            offset += line_str.len() + 1;
        }
        let char_offset: usize = line_str.chars().take(col_i).map(|c| c.len_utf8()).sum();
        offset + char_offset
    }

    pub fn move_cursor_up(&mut self) {
        let (line, col) = self.cursor_line_col();
        if line > 0 {
            let target_line = line - 1;
            let target_col = self.input_preferred_col.max(col);
            self.input_cursor = self.byte_offset_for_line_col(target_line, target_col);
        }
    }

    pub fn move_cursor_down(&mut self) {
        let (line, col) = self.cursor_line_col();
        let total_lines = self.input_buffer.split('\n').count();
        if line + 1 < total_lines {
            let target_line = line + 1;
            let target_col = self.input_preferred_col.max(col);
            self.input_cursor = self.byte_offset_for_line_col(target_line, target_col);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.input_cursor > 0 {
            let prev = self.input_buffer[..self.input_cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.input_cursor = prev;
            let (_, col) = self.cursor_line_col();
            self.input_preferred_col = col;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.input_cursor < self.input_buffer.len() {
            let next = self.input_buffer[self.input_cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.input_cursor + i)
                .unwrap_or(self.input_buffer.len());
            self.input_cursor = next;
            let (_, col) = self.cursor_line_col();
            self.input_preferred_col = col;
        }
    }

    pub fn move_line_start(&mut self) {
        let (line, _) = self.cursor_line_col();
        self.input_cursor = self.byte_offset_for_line_col(line, 0);
        self.input_preferred_col = 0;
    }

    pub fn move_line_end(&mut self) {
        let (line, _) = self.cursor_line_col();
        let lines: Vec<&str> = self.input_buffer.split('\n').collect();
        let line_len = lines.get(line).map(|l| l.chars().count()).unwrap_or(0);
        self.input_cursor = self.byte_offset_for_line_col(line, line_len);
        self.input_preferred_col = line_len;
    }

    pub fn delete_line(&mut self) {
        let (line, _) = self.cursor_line_col();
        let mut lines: Vec<String> = self
            .input_buffer
            .split('\n')
            .map(|s| s.to_string())
            .collect();
        if lines.is_empty() {
            return;
        }
        if line < lines.len() {
            lines.remove(line);
        }
        let new_len = lines.len();
        self.input_buffer = lines.join("\n");
        let new_line = line.min(new_len.saturating_sub(1));
        self.input_cursor = self.byte_offset_for_line_col(new_line, 0);
    }

    pub fn delete_char_at_cursor(&mut self) {
        if self.input_cursor < self.input_buffer.len() {
            let next = self.input_buffer[self.input_cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.input_cursor + i)
                .unwrap_or(self.input_buffer.len());
            self.input_buffer.drain(self.input_cursor..next);
        }
    }

    pub fn delete_word_before_cursor(&mut self) {
        if self.input_cursor == 0 {
            return;
        }
        let text = &self.input_buffer[..self.input_cursor];
        let mut char_indices = text.char_indices().rev().peekable();
        while let Some(&(_, c)) = char_indices.peek() {
            if c.is_whitespace() {
                char_indices.next();
            } else {
                break;
            }
        }
        let mut start_offest = 0;
        while let Some(&(i, c)) = char_indices.peek() {
            if !c.is_whitespace() {
                start_offest = i;
                char_indices.next();
            } else {
                break;
            }
        }

        self.input_buffer.drain(start_offest..self.input_cursor);
        self.input_cursor = start_offest;
        let (_, col) = self.cursor_line_col();
        self.input_preferred_col = col;
    }

    pub fn input_push(&mut self, c: char) {
        let idx = self.input_cursor;
        self.input_buffer.insert(idx, c);
        self.input_cursor += c.len_utf8();
        let (_, col) = self.cursor_line_col();
        self.input_preferred_col = col;
    }

    pub fn input_backspace(&mut self) {
        if self.input_cursor == 0 {
            return;
        }
        let end = self.input_cursor;
        let start = self.input_buffer[..end]
            .char_indices()
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.input_buffer.drain(start..end);
        self.input_cursor = start;
        let (_, col) = self.cursor_line_col();
        self.input_preferred_col = col;
    }

    pub fn input_clear(&mut self) {
        self.input_buffer.clear();
        self.input_cursor = 0;
        self.input_preferred_col = 0;
        self.input_edit_mode = InputEditMode::Insert;
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_line_col() {
        let mut app = App::new(vec![], Theme::default());
        app.input_buffer = "hello\nworld\nrust".to_string();
        app.input_cursor = 0;
        assert_eq!(app.cursor_line_col(), (0, 0));
        app.input_cursor = 6;
        assert_eq!(app.cursor_line_col(), (1, 0));
        app.input_cursor = 10;
        assert_eq!(app.cursor_line_col(), (1, 4));
    }

    #[test]
    fn test_move_cursor_up_down() {
        let mut app = App::new(vec![], Theme::default());
        app.input_buffer = "line1\nline2\nline3".to_string();
        app.input_cursor = 12;
        assert_eq!(app.cursor_line_col(), (2, 0));
        app.move_cursor_up();
        assert_eq!(app.cursor_line_col(), (1, 0));
        app.move_cursor_up();
        assert_eq!(app.cursor_line_col(), (0, 0));
        app.move_cursor_up();
        assert_eq!(app.cursor_line_col(), (1, 0));
    }

    #[test]
    fn test_delete_word_before_cursor() {
        let mut app = App::new(vec![], Theme::default());
        app.input_buffer = "hello world rust".to_string();
        app.input_cursor = app.input_buffer.len();
        app.delete_word_before_cursor();
        assert_eq!(app.input_buffer, "hello world ");
        app.delete_word_before_cursor();
        assert_eq!(app.input_buffer, "hello ");
        app.delete_word_before_cursor();
        assert_eq!(app.input_buffer, "");
    }
}
