use std::collections::HashMap;
use std::path::PathBuf;

use crate::models::stats::CardSM2State;

pub struct StatsStore {
    pub states: HashMap<String, CardSM2State>,
    pub path: PathBuf,
}

fn default_stats_path() -> PathBuf {
    directories::ProjectDirs::from("com", "fcard", "fcard")
        .map(|dirs| dirs.config_dir().join("stats.json"))
        .unwrap_or_else(|| PathBuf::from(".fcard/stats.json"))
}

impl StatsStore {
    pub fn load() -> Result<Self, String> {
        let path = default_stats_path();
        if !path.exists() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("create dirs {}: {}", parent.display(), e))?;
            }
            let store = StatsStore {
                states: HashMap::new(),
                path,
            };
            store.save()?;
            return Ok(store);
        }
        let raw = std::fs::read_to_string(&path)
            .map_err(|e| format!("read {}: {}", path.display(), e))?;
        let states: HashMap<String, CardSM2State> =
            serde_json::from_str(&raw).map_err(|e| format!("parse {}: {}", path.display(), e))?;
        Ok(StatsStore { states, path })
    }

    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.states)
            .map_err(|e| format!("serialize stats: {}", e))?;
        std::fs::write(&self.path, json)
            .map_err(|e| format!("write {}: {}", self.path.display(), e))
    }

    pub fn get_or_default(&self, card_id: &str) -> CardSM2State {
        self.states
            .get(card_id)
            .cloned()
            .unwrap_or_else(|| CardSM2State::new(card_id.to_string()))
    }

    pub fn update(&mut self, state: CardSM2State) -> Result<(), String> {
        self.states.insert(state.card_id.clone(), state);
        self.save()
    }

    pub fn due_ids(&self, current_epoch: i64) -> Vec<String> {
        self.states
            .values()
            .filter(|s| s.next_review_epoch <= current_epoch)
            .map(|s| s.card_id.clone())
            .collect()
    }

    #[allow(dead_code)]
    pub fn overall_mastery(&self) -> f64 {
        if self.states.is_empty() {
            return 0.0;
        }
        let total_correct: u32 = self.states.values().map(|s| s.total_correct).sum();
        let total_reviews: u32 = self
            .states
            .values()
            .map(|s| s.total_correct + s.total_incorrect)
            .sum();
        if total_reviews == 0 {
            return 0.0;
        }
        (total_correct as f64 / total_reviews as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::stats::CardSM2State;
    use std::path::PathBuf;

    fn make_store(states: HashMap<String, CardSM2State>) -> StatsStore {
        StatsStore {
            states,
            path: PathBuf::from("/dev/null"),
        }
    }

    #[test]
    fn get_or_default_returns_existing() {
        let mut map = HashMap::new();
        let mut s = CardSM2State::new("abc".to_string());
        s.total_correct = 5;
        map.insert("abc".to_string(), s.clone());
        let store = make_store(map);
        let result = store.get_or_default("abc");
        assert_eq!(result.total_correct, 5);
        assert_eq!(result.card_id, "abc");
    }

    #[test]
    fn get_or_default_returns_default_for_unknown() {
        let store = make_store(HashMap::new());
        let result = store.get_or_default("unknown");
        assert_eq!(result.card_id, "unknown");
        assert_eq!(result.repetitions, 0);
        assert_eq!(result.total_correct, 0);
        assert!((result.ease_factor - 2.5).abs() < f32::EPSILON);
    }
    #[test]
    fn overall_mastery_empty() {
        let store = make_store(HashMap::new());
        assert_eq!(store.overall_mastery(), 0.0)
    }

    #[test]
    fn overall_mastery_no_reviews() {
        let mut map = HashMap::new();
        map.insert("a".to_string(), CardSM2State::new("a".to_string()));
        let store = make_store(map);
        assert_eq!(store.overall_mastery(), 0.0);
    }

    #[test]
    fn overall_mastery_mixed() {
        let mut map = HashMap::new();
        let mut s1 = CardSM2State::new("a".to_string());
        s1.total_correct = 3;
        s1.total_incorrect = 1;
        let mut s2 = CardSM2State::new("b".to_string());
        s2.total_correct = 7;
        s2.total_incorrect = 3;

        map.insert("a".to_string(), s1);
        map.insert("b".to_string(), s2);
        let store = make_store(map);
        let mastery = store.overall_mastery();
        assert!((mastery - (10.0 / 14.0 * 100.0)).abs() < 1e-9);
    }

    #[test]
    fn due_ids_filters_correctly() {
        let mut map = HashMap::new();

        let mut past = CardSM2State::new("past".to_string());
        past.next_review_epoch = 1000;
        let mut future = CardSM2State::new("future".to_string());
        future.next_review_epoch = 9_999_999;
        let mut now = CardSM2State::new("now".to_string());
        now.next_review_epoch = 5000;

        map.insert("past".to_string(), past);
        map.insert("future".to_string(), future);
        map.insert("now".to_string(), now);
        let store = make_store(map);
        let mut due = store.due_ids(5000);
        due.sort();
        assert_eq!(due, vec!["now", "past"]);
    }
}
