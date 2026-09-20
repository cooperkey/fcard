pub fn normalize(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
pub fn score(answer: &str, expected: &str) -> f64 {
    let norm_ans = normalize(answer);
    let norm_exp = normalize(expected);

    if norm_ans.is_empty() && norm_exp.is_empty() {
        return 1.0;
    }
    if norm_ans.is_empty() || norm_exp.is_empty() {
        return 0.0;
    }

    let ops = compute_lcs_diff(&norm_ans, &norm_exp);
    let mut match_chars = 0;
    for op in ops {
        if let DiffOp::Equal(s) = op {
            match_chars += s.chars().count();
        }
    }

    let max_len = norm_ans.chars().count().max(norm_exp.chars().count());
    if max_len == 0 {
        1.0
    } else {
        match_chars as f64 / max_len as f64
    }
}

pub const MATCH_THRESHOLD: f64 = 0.75;

pub fn is_match(answer: &str, expected: &str) -> bool {
    score(answer, expected) >= MATCH_THRESHOLD
}

pub fn feedback(score: f64) -> &'static str {
    if score >= 0.90 {
        "Correct"
    } else if score >= 0.75 {
        "Close enough"
    } else if score >= 0.50 {
        "Almost - check spelling"
    } else {
        "Incorrect"
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffOp {
    Equal(String),
    Delete(String),
    Insert(String),
}

pub fn compute_lcs_diff(user_text: &str, expected_text: &str) -> Vec<DiffOp> {
    let u_chars: Vec<char> = user_text.chars().collect();
    let e_chars: Vec<char> = expected_text.chars().collect();

    let m = u_chars.len();
    let n = e_chars.len();

    let u_lower: Vec<char> = u_chars.iter().flat_map(|c| c.to_lowercase()).collect();
    let e_lower: Vec<char> = e_chars.iter().flat_map(|c| c.to_lowercase()).collect();

    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in (0..m).rev() {
        for j in (0..n).rev() {
            if u_lower[i] == e_lower[j] {
                dp[i][j] = 1 + dp[i + 1][j + 1];
            } else {
                dp[i][j] = dp[i + 1][j].max(dp[i][j + 1]);
            }
        }
    }

    let mut i = 0;
    let mut j = 0;
    let mut raw_ops = Vec::new();

    while i < m || j < n {
        if i < m && j < n && u_lower[i] == e_lower[j] {
            raw_ops.push(DiffOp::Equal(u_chars[i].to_string()));
            i += 1;
            j += 1;
        } else if j < n && (i == m || dp[i][j + 1] >= dp[i + 1][j]) {
            raw_ops.push(DiffOp::Insert(e_chars[j].to_string()));
            j += 1;
        } else if i < m {
            raw_ops.push(DiffOp::Delete(u_chars[i].to_string()));
            i += 1;
        }
    }

    let mut merged: Vec<DiffOp> = Vec::new();
    for op in raw_ops {
        match (merged.last_mut(), op) {
            (Some(DiffOp::Equal(cur)), DiffOp::Equal(next)) => cur.push_str(&next),
            (Some(DiffOp::Delete(cur)), DiffOp::Delete(next)) => cur.push_str(&next),
            (Some(DiffOp::Insert(cur)), DiffOp::Insert(next)) => cur.push_str(&next),
            (_, new_op) => merged.push(new_op),
        }
    }
    merged
}

pub fn diff_user_answer_multiline<'a>(
    user_answer: &str,
    expected: &str,
    normal_style: ratatui::style::Style,
    success_style: ratatui::style::Style,
    error_style: ratatui::style::Style,
) -> Vec<ratatui::text::Line<'a>> {
    let ops = compute_lcs_diff(user_answer, expected);

    let mut result_lines = Vec::new();
    let mut current_spans: Vec<ratatui::text::Span<'a>> = vec![ratatui::text::Span::raw(" > ")];

    for op in ops {
        let (text, style) = match op {
            DiffOp::Equal(s) => (s, normal_style),
            DiffOp::Insert(s) => (s, success_style),
            DiffOp::Delete(s) => (s, error_style),
        };

        let parts: Vec<&str> = text.split('\n').collect();
        for (idx, part) in parts.iter().enumerate() {
            if idx > 0 {
                result_lines.push(ratatui::text::Line::from(std::mem::take(
                    &mut current_spans,
                )));
                current_spans.push(ratatui::text::Span::raw("   "));
            }
            if !part.is_empty() {
                current_spans.push(ratatui::text::Span::styled((*part).to_string(), style));
            }
        }
    }

    if !current_spans.is_empty() {
        result_lines.push(ratatui::text::Line::from(current_spans));
    }
    result_lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match() {
        assert!(is_match("photosynthesis", "photosynthesis"));
        assert_eq!(
            feedback(score("photosynthesis", "photosynthesis")),
            "Correct"
        );
    }
    #[test]
    fn match_with_typo() {
        assert!(is_match("photosinthesis", "photosynthesis"));
    }

    #[test]
    fn match_case_and_punctuation() {
        assert!(is_match("  Photosynthesis!  ", "photosynthesis"));
        assert!(is_match("Hello, World.", "hellow world"));
    }

    #[test]
    fn clear_mismatch() {
        assert!(!is_match("mitosis", "photosynthesis"));
        assert_eq!(feedback(score("mitosis", "photosynthesis")), "Incorrect");
    }

    #[test]
    fn normalize_collapses_whitespace() {
        assert_eq!(normalize("  foo  bar  "), "foo bar");
    }

    #[test]
    fn normalize_strips_punctuation() {
        assert_eq!(normalize("it's a test!"), "its a test");
    }

    #[test]
    fn feedback_band() {
        assert_eq!(feedback(1.00), "Correct");
        assert_eq!(feedback(0.90), "Correct");
        assert_eq!(feedback(0.85), "Close Enough");
        assert_eq!(feedback(0.75), "Close Enough");
        assert_eq!(feedback(0.70), "Almost - check spellling");
        assert_eq!(feedback(0.50), "Almost - check spellling");
        assert_eq!(feedback(0.49), "Incorrect");
        assert_eq!(feedback(0.00), "Incorrect");
    }

    #[test]
    fn test_comute_lcs_diff() {
        let ops = compute_lcs_diff("Alto", "Alto/Contralto");
        assert_eq!(
            ops,
            vec![
                DiffOp::Equal("Alto".to_string()),
                DiffOp::Insert("/Contralto".to_string()),
            ]
        );
    }
}
