use crate::models::card::{Card, Deck, DeckFormat};
use std::fs;
use std::path::Path;

pub fn detect_format(path: &Path) -> Result<DeckFormat, String> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => Ok(DeckFormat::Json),
        Some("yaml") | Some("yml") => Ok(DeckFormat::Yaml),
        Some("md") | Some("markdown") => Ok(DeckFormat::Markdown),
        _ => Err(format![
            "unsupported file extension: {:?}",
            path.extension()
        ]),
    }
}

pub fn has_fcard_marker(path: &Path) -> bool {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let lower = content.to_lowercase();
    lower.contains("#fcard")
        || lower.contains("# fcard")
        || lower.contains("<!-- fcard")
        || lower.contains("<!--#fcard")
        || lower.contains("<!-- #fcard")
}

pub fn load_deck(path: &Path) -> Result<Deck, String> {
    let format = detect_format(path)?;
    let mut deck = match format {
        DeckFormat::Json => load_json(path)?,
        DeckFormat::Yaml => load_yaml(path)?,
        DeckFormat::Markdown => load_markdown(path)?,
    };

    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();
    deck.id = file_stem;
    deck.file_path = path.to_path_buf();
    deck.format = format;

    for (i, card) in deck.cards.iter_mut().enumerate() {
        if card.id.is_empty() {
            card.id = format!("{}-{}", deck.id, i);
        }
    }

    Ok(deck)
}

fn load_json(path: &Path) -> Result<Deck, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let sanitized: String = content
        .lines()
        .filter(|line| !line.trim_start().starts_with('#') && !line.trim_start().starts_with("//"))
        .collect::<Vec<&str>>()
        .join("\n");
    serde_json::from_str(&sanitized).map_err(|e| e.to_string())
}

fn load_yaml(path: &Path) -> Result<Deck, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_yaml::from_str(&content).map_err(|e| e.to_string())
}

fn load_markdown(path: &Path) -> Result<Deck, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    parse_markdown_str(&content)
}

fn parse_markdown_str(content: &str) -> Result<Deck, String> {
    let mut lines = content.lines().peekable();
    let mut title = String::new();
    let mut description_lines = Vec::new();

    while let Some(&line) = lines.peek() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            break;
        }
        lines.next();

        if trimmed.is_empty() || trimmed == "##fcard" || trimmed == "#fcard" {
            continue;
        }

        if trimmed.starts_with("# ") && title.is_empty() {
            let candidate = trimmed.trim_start_matches("# ").trim();
            if candidate.to_lowercase() != "#fcard" && candidate.to_lowercase() != "fcard" {
                title = candidate.to_string();
            }
        } else if !title.is_empty() {
            description_lines.push(trimmed);
        }
    }

    let description = if description_lines.is_empty() {
        None
    } else {
        Some(description_lines.join("\n").trim().to_string())
    };

    let mut cards = Vec::new();
    let mut current_card_lines: Vec<&str> = Vec::new();
    let mut current_h2_title = String::new();

    let flush_card = |h2_title: &str, card_lines: &[&str], cards_vec: &mut Vec<Card>| {
        if h2_title.is_empty() && card_lines.is_empty() {
            return;
        }
        let card = parse_card_block(h2_title, card_lines);
        if !card.front.is_empty() || !card.back.is_empty() {
            cards_vec.push(card);
        }
    };

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            flush_card(&current_h2_title, &current_card_lines, &mut cards);
            current_card_lines.clear();
            current_h2_title = trimmed.trim_start_matches("## ").trim().to_string();
        } else {
            current_card_lines.push(line);
        }
    }
    flush_card(&current_h2_title, &current_card_lines, &mut cards);

    Ok(Deck {
        id: String::new(),
        title: if title.is_empty() {
            "untitled deck".to_string()
        } else {
            title
        },
        description,
        file_path: std::path::PathBuf::new(),
        format: DeckFormat::Markdown,
        cards,
    })
}

fn parse_key_value_line(line: &str) -> Option<(&'static str, String)> {
    let trimmed = line.trim();
    let clean = trimmed
        .replace("**", "")
        .replace("__", "")
        .replace(['*', '_'], "");
    let clean_trimmed = clean.trim();

    let (key, val) = clean_trimmed.split_once(':')?;
    let key_lower = key.trim().to_lowercase();
    let val_str = val.trim().to_string();

    match key_lower.as_str() {
        "front" | "q" | "question" => Some(("front", val_str)),
        "back" | "a" | "answer" => Some(("back", val_str)),
        "hint" | "hints" => Some(("hints", val_str)),
        "tag" | "tags" => Some(("tags", val_str)),
        _ => None,
    }
}

fn parse_card_block(h2_title: &str, lines: &[&str]) -> Card {
    let mut front_lines: Vec<String> = Vec::new();
    let mut back_lines: Vec<String> = Vec::new();
    let mut hints: Vec<String> = Vec::new();
    let mut tags: Vec<String> = Vec::new();

    enum FieldTarget {
        None,
        Front,
        Back,
    }

    let mut current_target = FieldTarget::None;
    let mut explicit_fields_found = false;

    for &line in lines {
        let trimmed = line.trim();

        if let Some((key, val)) = parse_key_value_line(trimmed) {
            explicit_fields_found = true;
            match key {
                "front" => {
                    current_target = FieldTarget::Front;
                    if !val.is_empty() {
                        front_lines.push(val);
                    }
                }
                "back" => {
                    current_target = FieldTarget::Back;
                    if !val.is_empty() {
                        back_lines.push(val);
                    }
                }
                "hints" => {
                    for h in val.split(',') {
                        let h_trim = h.trim();
                        if !h_trim.is_empty() {
                            hints.push(h_trim.to_string());
                        }
                    }
                }
                "tags" => {
                    for t in val.split(',') {
                        let t_trim = t.trim();
                        if !t_trim.is_empty() {
                            tags.push(t_trim.to_string());
                        }
                    }
                }
                _ => {}
            }
        } else if trimmed.starts_with("### ") {
            let h3 = trimmed.trim_start_matches("### ").trim().to_lowercase();
            if h3.contains("front") || h3.contains("question") || h3.contains("prompt") {
                explicit_fields_found = true;
                current_target = FieldTarget::Front;
            } else if h3.contains("back") || h3.contains("answer") || h3.contains("solution") {
                explicit_fields_found = true;
                current_target = FieldTarget::Back;
            } else {
                current_target = FieldTarget::None;
            }
        } else {
            match current_target {
                FieldTarget::Front => {
                    front_lines.push(line.to_string());
                }
                FieldTarget::Back => {
                    back_lines.push(line.to_string());
                }
                FieldTarget::None => {
                    if !explicit_fields_found && !trimmed.is_empty() {
                        back_lines.push(line.to_string());
                    }
                }
            }
        }
    }

    let front = if !front_lines.is_empty() {
        front_lines.join("\n").trim().to_string()
    } else if !explicit_fields_found && !h2_title.is_empty() {
        h2_title.trim().to_string()
    } else {
        String::new()
    };

    let back = back_lines.join("\n").trim().to_string();

    Card {
        id: String::new(),
        front,
        back,
        hints,
        tags,
    }
}

pub fn save_deck(deck: &Deck) -> Result<(), String> {
    let path = &deck.file_path;
    if path.as_os_str().is_empty() {
        return Err("cannot save deck with empty file path".to_string());
    }
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    match deck.format {
        DeckFormat::Json => {
            let json = serde_json::to_string_pretty(deck).map_err(|e| e.to_string())?;
            let formatted = format!("#fcard\n{}", json);
            fs::write(path, formatted).map_err(|e| e.to_string())
        }
        DeckFormat::Yaml => {
            let yaml = serde_yaml::to_string(deck).map_err(|e| e.to_string())?;
            let formatted = format!("# #fcard\n{}", yaml);
            fs::write(path, formatted).map_err(|e| e.to_string())
        }
        DeckFormat::Markdown => {
            let mut md = String::new();
            md.push_str("# #fcard\n");
            md.push_str(&format!("# {}\n", deck.title));
            if let Some(desc) = &deck.description {
                md.push_str(&format!("{}\n", desc));
            }
            md.push('\n');
            for (i, card) in deck.cards.iter().enumerate() {
                md.push_str(&format!("## Card {}\n", i + 1));
                md.push_str(&format!("Front: {}\n", card.front));
                md.push_str(&format!("Back: {}\n", card.back));
                if !card.hints.is_empty() {
                    md.push_str(&format!("Hints: {}\n", card.hints.join(", ")));
                }
                if !card.tags.is_empty() {
                    md.push_str(&format!("tags: {}\n", card.tags.join(", ")));
                }
                md.push('\n');
            }
            fs::write(path, md).map_err(|e| e.to_string())
        }
    }
}

pub fn write_template(path: &Path, format: DeckFormat) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    match format {
        DeckFormat::Json => {
            let template = r#"#fcard
{
    "title": "Deck Title Here",
    "description": "Add description here.",
    "cards": [
        {
            "front": "Write your question or prompt here.",
            "back": "Write the answer or solution here.",
            "hints": [
                "Optional hints here."
            ],
            "tags": [
                "topic1",
                "topic2"
            ],
        },
        {
            "front": "Write your question or prompt here.",
            "back": "Write the answer or solution here.",
            "hints": [],
            "tags": [
                "basics"
            ]
        }
    ]
}"#;

            fs::write(path, template).map_err(|e| e.to_string())
        }
        DeckFormat::Yaml => {
            let template = r#"# #fcard
title: Deck Title Here
description: Add description here.
cards:
    - front: Write your question or prompt here.
    - back: Write the answer or solution here.
    - hints:
        - Optional hints here.
    - tags:
        - topic1
        - topic2
    - front: Write your question or promt here.
    - back: Write the answer or solution here.
    - hints: []
    - tags:
        - basics
"#;
            fs::write(path, template).map_err(|e| e.to_string())
        }
        DeckFormat::Markdown => {
            let template = r#" #fcard
# Deck Title Here
Add description here.

## Sample Card 1
Front: Write your question or prompt here.
Back: Write the answer or solution here.
Hints: Optional hints herer.
Tags: topic1, topic2

## Sample card 2
Front: Write your question or prompt here.
Back: Write the answer or solution here.
Tags: basics
"#;
            fs::write(path, template).map_err(|e| e.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format() {
        assert_eq!(
            detect_format(Path::new("test.json")).unwrap(),
            DeckFormat::Json
        );
        assert_eq!(
            detect_format(Path::new("test.yaml")).unwrap(),
            DeckFormat::Yaml
        );
        assert_eq!(
            detect_format(Path::new("test.yml")).unwrap(),
            DeckFormat::Yaml
        );
        assert_eq!(
            detect_format(Path::new("test.md")).unwrap(),
            DeckFormat::Markdown
        );
        assert_eq!(
            detect_format(Path::new("test.markdown")).unwrap(),
            DeckFormat::Markdown
        );
        assert!(detect_format(Path::new("test.txt")).is_err());
    }

    #[test]
    fn test_save_deck_json_roundtrip() {
        let dir = std::env::temp_dir();
        let path = dir.join("fcard_test_save.json");
        let deck = Deck {
            id: "test".to_string(),
            title: "Test Deck".to_string(),
            description: Some("Description".to_string()),
            file_path: path.clone(),
            format: DeckFormat::Json,
            cards: vec![Card {
                id: "test-0".to_string(),
                front: "Q1".to_string(),
                back: "A1".to_string(),
                hints: vec![],
                tags: vec![],
            }],
        };

        save_deck(&deck).unwrap();
        let loaded = load_deck(&path).unwrap();
        assert_eq!(loaded.cards.len(), 1);
        assert_eq!(loaded.cards[0].front, "Q1");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_has_fcard_marker() {
        let dir = std::env::temp_dir();
        let path = dir.join("fcard_marker_test.md");
        fs::write(&path, "# #lazycard\n# Sample Deck\n").unwrap();
        assert!(has_fcard_marker(&path));

        let path2 = dir.join("fcard_no_marker_test.md");
        fs::write(&path2, "# Sample Deck\nNo tag here\n").unwrap();
        assert!(!has_fcard_marker(&path2));
        let _ = fs::remove_file(path);
        let _ = fs::remove_file(path2);
    }

    #[test]
    fn test_load_json_with_header_commint() {
        let dir = std::env::temp_dir();
        let path = dir.join("fcard_comment_test.json");
        let content = "#fcard\n{\n  \"title\":\"Test\",\n  \"cards\": []\n}";
        fs::write(&path, content).unwrap();
        let deck = load_deck(&path).unwrap();
        assert_eq!(deck.title, "Test");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_write_template_has_fcard_marker_and_loads() {
        let dir = std::env::temp_dir();
        let json_path = dir.join("template_test.json");
        write_template(&json_path, DeckFormat::Json).unwrap();
        assert!(has_fcard_marker(&json_path));
        let json_deck = load_deck(&json_path).unwrap();
        assert_eq!(json_deck.cards.len(), 2);
        assert!(json_deck.cards[0].front.is_empty());
        assert!(json_deck.cards[0].back.is_empty());
        let _ = fs::remove_file(json_path);

        let dir = std::env::temp_dir();
        let yaml_path = dir.join("template_test.yaml");
        write_template(&yaml_path, DeckFormat::Yaml).unwrap();
        assert!(has_fcard_marker(&yaml_path));
        let yaml_deck = load_deck(&yaml_path).unwrap();
        assert_eq!(yaml_deck.cards.len(), 2);
        assert!(yaml_deck.cards[0].front.is_empty());
        assert!(yaml_deck.cards[0].back.is_empty());
        let _ = fs::remove_file(yaml_path);

        let md_path = dir.join("template_test.md");
        write_template(&md_path, DeckFormat::Markdown).unwrap();
        assert!(has_fcard_marker(&md_path));
        let md_deck = load_deck(&md_path).unwrap();
        assert_eq!(md_deck.cards.len(), 2);
        assert_eq!(
            md_deck.cards[0].front,
            "Write your question or prompt here."
        );
        assert_eq!(md_deck.cards[0].back, "Write the answer or solution here.");
        assert_eq!(md_deck.cards[1].front, "Another question");
        assert_eq!(md_deck.cards[1].back, "Another answer");
        let _ = fs::remove_file(md_path);
    }

    #[test]
    fn test_load_markdown_variations() {
        let dir = std::env::temp_dir();
        let md_b = dir.join("md_format_b.md");
        let content_b = "# #fcard\n# Test quiz\n\n## What is the capital of France?\nParis\n\n## What is the capital of Japan?\nJ\n";
        fs::write(&md_b, content_b).unwrap();
        let deck_b = load_deck(&md_b).unwrap();
        assert_eq!(deck_b.cards.len(), 2);
        assert_eq!(deck_b.cards[0].front, "What is the capital of France?");
        assert_eq!(deck_b.cards[0].back, "Paris");
        assert_eq!(deck_b.cards[1].front, "What is the capital of Japan?");
        assert_eq!(deck_b.cards[1].back, "J");
        let _ = fs::remove_file(md_b);
    }
}
