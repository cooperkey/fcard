use crate::models::stats::CardSM2State;

pub fn update_sm2(state: &mut CardSM2State, quality_rating: u8, current_epoch: i64) {
    let q = quality_rating.min(5) as f32;

    if q >= 3.0 {
        match state.repetitions {
            0 => state.interval_days = 1,
            1 => state.interval_days = 6,
            _ => {
                state.interval_days =
                    (state.interval_days as f32 * state.ease_factor).round() as u32;
            }
        }
        state.repetitions += 1;
        state.total_correct += 1;
    } else {
        state.repetitions = 0;
        state.interval_days = 1;
        state.total_incorrect += 1;
    }

    let new_ef = state.ease_factor + (0.1 - (5.0 - q) * (0.08 + (5.0 - q) * 0.02));
    state.ease_factor = new_ef.max(1.3);

    let seconds_per_day = 86400;
    state.last_reviewed_epoch = Some(current_epoch);
    state.next_review_epoch = current_epoch + (state.interval_days as i64 * seconds_per_day);
}

#[allow(dead_code)]
pub fn is_due(state: &CardSM2State, current_epoch: i64) -> bool {
    state.next_review_epoch <= current_epoch
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sm2_perfect_response() {
        let mut state = CardSM2State::new("card1".to_string());
        let epoch = 1000;

        update_sm2(&mut state, 5, epoch);
        assert_eq!(state.repetitions, 1);
        assert_eq!(state.interval_days, 1);
        assert!(state.ease_factor > 2.5);
        assert_eq!(state.next_review_epoch, epoch + 86400);

        update_sm2(&mut state, 5, epoch + 86400);
        assert_eq!(state.repetitions, 2);
        assert_eq!(state.interval_days, 6);

        let old_ef = state.ease_factor;
        update_sm2(&mut state, 5, epoch + 86400 * 7);
        assert_eq!(state.repetitions, 3);
        assert_eq!(state.interval_days, (6.0 * old_ef).round() as u32);
    }

    #[test]
    fn test_sm2_failure_resets() {
        let mut state = CardSM2State::new("card1".to_string());
        state.repetitions = 5;
        state.interval_days = 20;
        state.ease_factor = 2.8;

        update_sm2(&mut state, 1, 1000);
        assert_eq!(state.repetitions, 0);
        assert_eq!(state.interval_days, 1);
        assert_eq!(state.total_incorrect, 1);
        assert!(state.ease_factor < 2.8);
    }
}
