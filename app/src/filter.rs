use crate::models::Candidate;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct CandidateFilter {
    positive_fields: HashMap<String, Vec<String>>,
    negative_fields: Vec<(String, String)>,
    positive_text: Vec<String>,
    negative_text: Vec<String>,
}

impl CandidateFilter {
    pub fn parse(input: &str) -> Self {
        let mut result = Self::default();
        for token in tokenize(input) {
            let (positive, body) = match token.chars().next() {
                Some('+') => (true, &token[1..]),
                Some('-') => (false, &token[1..]),
                _ => (true, token.as_str()),
            };
            if body.is_empty() {
                continue;
            }
            if let Some((field, value)) = body.split_once(':') {
                let pair = (field.to_lowercase(), value.to_lowercase());
                if positive {
                    result.positive_fields.entry(pair.0).or_default().push(pair.1);
                } else {
                    result.negative_fields.push(pair);
                }
            } else if positive {
                result.positive_text.push(body.to_lowercase());
            } else {
                result.negative_text.push(body.to_lowercase());
            }
        }
        result
    }

    pub fn matches(&self, candidate: &Candidate, recorded: bool) -> bool {
        let text = candidate.searchable_text(recorded);
        if self.positive_text.iter().any(|value| !text.contains(value)) {
            return false;
        }
        if self.negative_text.iter().any(|value| text.contains(value)) {
            return false;
        }
        for (field, values) in &self.positive_fields {
            if !values.iter().any(|value| field_matches(candidate, recorded, field, value, &text)) {
                return false;
            }
        }
        !self
            .negative_fields
            .iter()
            .any(|(field, value)| field_matches(candidate, recorded, field, value, &text))
    }
}

fn field_matches(candidate: &Candidate, recorded: bool, field: &str, value: &str, text: &str) -> bool {
    let contains = |source: &str| source.to_lowercase().contains(value);
    match field {
        "map" => contains(&candidate.map_name),
        "class" => contains(&candidate.attacker_class),
        "team" => {
            let normalized = if value == "blue" { "blu" } else { value };
            candidate.attacker_team.to_lowercase() == normalized
        }
        "demo" => contains(&candidate.source_demo),
        "mode" => contains(&candidate.demo_context.mode) || contains(&candidate.demo_context.mode_label),
        "type" | "demo_type" | "capture" => contains(&candidate.demo_context.capture_type),
        "tag" => candidate.tags.iter().any(|tag| contains(tag)),
        "player" => value.trim_start_matches('#').parse::<i64>().ok() == Some(candidate.attacker_user_id),
        "recorded" => match value {
            "true" | "yes" | "1" | "recorded" => recorded,
            "false" | "no" | "0" | "unrecorded" => !recorded,
            _ => false,
        },
        "weapon" => candidate.kills.iter().any(|kill| {
            kill.get("weapon")
                .and_then(serde_json::Value::as_str)
                .is_some_and(contains)
        }),
        "text" => text.contains(value),
        _ => false,
    }
}

fn tokenize(input: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    for character in input.chars() {
        match character {
            '"' => quoted = !quoted,
            character if character.is_whitespace() && !quoted => {
                if !current.is_empty() {
                    result.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(character),
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_field_is_or_and_different_fields_are_and() {
        let candidate = Candidate {
            attacker_class: "demoman".into(),
            map_name: "koth_product_final".into(),
            ..Candidate::default()
        };
        assert!(CandidateFilter::parse("+class:soldier +class:demoman +map:product").matches(&candidate, false));
        assert!(!CandidateFilter::parse("+class:demoman +map:steel").matches(&candidate, false));
    }
}
