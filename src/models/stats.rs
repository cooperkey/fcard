use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardSM2State {
    pub card_id: String,
    pub repetitions: u32,
    pub interval_days: u32,
    pub ease_factor: f32,
    pub next_review_epoch: i64,
    pub last_reviewed_epoch: Option<i64>,
    pub total_correct: u32,
    pub total_incorrect: u32,
}

impl Default for CardSM2State {
    fn default() -> Self {
        Self {
            card_id: String::new(),
            repetitions: 0,
            interval_days: 0,
            ease_factor: 2.5,
            next_review_epoch: 0,
            last_reviewed_epoch: None,
            total_correct: 0,
            total_incorrect: 0,
        }
    }
}

impl CardSM2State {
    pub fn new(card_id: String) -> Self {
        Self {
            card_id,
            ..Default::default()
        }
    }

    #[allow(dead_code)]
    pub fn mastery_percentage(&self) -> f64 {
        let total = self.total_correct + self.total_incorrect;
        if total == 0 {
            return 0.0;
        }
        (self.total_correct as f64 / total as f64) * 100.0
    }
}
