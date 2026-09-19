use crate::models::*;
use anyhow::{Result, anyhow};
use regex::Regex;
use std::collections::HashMap;
use tracing::debug;

pub struct PatternEngine {
    // Cache for compiled regex patterns (simple fallback)
    pattern_cache: HashMap<String, Regex>,
}

impl PatternEngine {
    pub fn new() -> Self {
        Self {
            pattern_cache: HashMap::new(),
        }
    }

    /// Validate code against a set of patterns
    pub fn validate_code(
        &self,
        code: &str,
        _language: &str,
        patterns: Vec<Pattern>,
    ) -> Result<Vec<PatternViolation>> {
        let mut violations = Vec::new();

        for pattern in patterns {
            if !pattern.enabled {
                continue;
            }

            // Find violations using regex (simplified version)
            // In production, use ast-grep-core for real AST-based matching
            if let Ok(matches) = self.find_pattern_matches(&pattern.rule, code) {
                debug!(
                    "Pattern '{}' found {} matches",
                    pattern.name,
                    matches.len()
                );

                for (line_num, column) in matches {
                    violations.push(PatternViolation {
                        pattern_id: pattern.id.clone(),
                        pattern_name: pattern.name.clone(),
                        severity: pattern.severity,
                        location: CodeLocation {
                            file: "code".to_string(),
                            line: line_num,
                            column,
                        },
                        message: format!("Pattern violation: {}", pattern.description),
                        suggestion: pattern.rewrite.clone(),
                    });
                }
            }
        }

        Ok(violations)
    }

    /// Find pattern matches in code (simplified regex-based)
    fn find_pattern_matches(&self, pattern: &str, code: &str) -> Result<Vec<(u32, u32)>> {
        // This is a simplified version using regex
        // In production, integrate with ast-grep-core for full AST analysis

        let regex = Regex::new(pattern)?;
        let mut matches = Vec::new();

        for (line_num, line) in code.lines().enumerate() {
            for capture in regex.captures_iter(line) {
                if let Some(matched) = capture.get(0) {
                    matches.push((
                        (line_num + 1) as u32,
                        matched.start() as u32,
                    ));
                }
            }
        }

        Ok(matches)
    }

    /// Validate a single pattern rule (check if it's valid)
    pub fn validate_pattern_rule(&self, rule: &str, language: &str) -> Result<()> {
        // Check if regex is valid
        Regex::new(rule)
            .map_err(|e| anyhow!("Invalid pattern rule for {}: {}", language, e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_matching() {
        let engine = PatternEngine::new();
        let code = "console.log('hello');\nconsole.log('world');";
        
        let pattern = Pattern::new(
            "console.log".to_string(),
            "Find console.log calls".to_string(),
            r"console\.log".to_string(),
            vec!["typescript".to_string()],
        );

        let violations = engine.validate_code(code, "typescript", vec![pattern]).unwrap();
        assert_eq!(violations.len(), 2);
    }
}
