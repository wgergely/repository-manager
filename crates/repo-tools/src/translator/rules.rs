//! Rule translation with semantic formatting
//!
//! This module translates rules into tool-specific instruction format,
//! RESPECTING the tool's declared capabilities.

use super::TranslatedContent;
use repo_meta::schema::{ConfigType, RuleDefinition, Severity, ToolDefinition};

/// Translates rules into instructions for tools.
///
/// KEY FEATURE: This translator CHECKS the tool's capabilities before
/// generating any output. If a tool doesn't support custom instructions,
/// no instructions are generated.
pub struct RuleTranslator;

impl RuleTranslator {
    /// Translate rules for a specific tool.
    ///
    /// Returns empty content if:
    /// - The tool doesn't support custom instructions
    /// - No rules are provided
    pub fn translate(tool: &ToolDefinition, rules: &[RuleDefinition]) -> TranslatedContent {
        // KEY: Actually check the capability!
        if !tool.capabilities.supports_custom_instructions {
            return TranslatedContent::empty();
        }

        if rules.is_empty() {
            return TranslatedContent::empty();
        }

        let format = tool.integration.config_type;
        let instructions = Self::format_rules(rules, format);
        TranslatedContent::with_instructions(format, instructions)
    }

    /// Format rules into a string.
    fn format_rules(rules: &[RuleDefinition], format: ConfigType) -> String {
        // Sort by severity (mandatory first)
        let mut sorted: Vec<_> = rules.iter().collect();
        sorted.sort_by_key(|r| match r.meta.severity {
            Severity::Mandatory => 0,
            Severity::Suggestion => 1,
        });

        sorted
            .iter()
            .map(|r| Self::format_rule(r, format))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Format a single rule based on config type.
    fn format_rule(rule: &RuleDefinition, format: ConfigType) -> String {
        match format {
            ConfigType::Markdown | ConfigType::Text => Self::format_markdown(rule),
            _ => rule.content.instruction.clone(),
        }
    }

    /// Format a rule as markdown with severity markers and examples.
    fn format_markdown(rule: &RuleDefinition) -> String {
        let marker = match rule.meta.severity {
            Severity::Mandatory => "**[REQUIRED]**",
            Severity::Suggestion => "[Suggested]",
        };

        let mut out = format!(
            "## {} {}\n\n{}",
            rule.meta.id, marker, rule.content.instruction
        );

        // Add examples if present
        if let Some(ref examples) = rule.examples {
            if !examples.positive.is_empty() {
                out.push_str("\n\n**Good:**\n");
                for example in &examples.positive {
                    out.push_str(&format!("```\n{}\n```\n", example));
                }
            }
            if !examples.negative.is_empty() {
                out.push_str("\n**Bad:**\n");
                for example in &examples.negative {
                    out.push_str(&format!("```\n{}\n```\n", example));
                }
            }
        }

        // Add file patterns if present
        if let Some(ref targets) = rule.targets
            && !targets.file_patterns.is_empty()
        {
            out.push_str(&format!(
                "\n\n**Applies to:** {}",
                targets.file_patterns.join(", ")
            ));
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_meta::schema::{
        RuleContent, RuleExamples, RuleMeta, RuleTargets, ToolCapabilities, ToolIntegrationConfig,
        ToolMeta,
    };

    fn make_tool(supports_instructions: bool) -> ToolDefinition {
        ToolDefinition {
            meta: ToolMeta {
                name: "Test".into(),
                slug: "test".into(),
                description: None,
            },
            integration: ToolIntegrationConfig {
                config_path: ".test".into(),
                config_type: ConfigType::Markdown,
                additional_paths: vec![],
            },
            capabilities: ToolCapabilities {
                supports_custom_instructions: supports_instructions,
                supports_mcp: false,
                supports_rules_directory: false,
            },
            schema_keys: None,
        }
    }

    fn make_rule(id: &str, severity: Severity) -> RuleDefinition {
        RuleDefinition {
            meta: RuleMeta {
                id: id.into(),
                severity,
                tags: vec![],
            },
            content: RuleContent {
                instruction: format!("Do {} things", id),
            },
            examples: None,
            targets: None,
        }
    }

    #[test]
    fn test_empty_when_no_capability() {
        let tool = make_tool(false);
        let rules = vec![make_rule("rule1", Severity::Mandatory)];

        let content = RuleTranslator::translate(&tool, &rules);
        assert!(content.is_empty());
    }

    #[test]
    fn test_translates_when_capable() {
        let tool = make_tool(true);
        let rules = vec![make_rule("rule1", Severity::Mandatory)];

        let content = RuleTranslator::translate(&tool, &rules);
        assert!(!content.is_empty());
        assert!(content.instructions.is_some());
        assert!(content.instructions.as_ref().unwrap().contains("rule1"));
    }

    #[test]
    fn test_empty_when_no_rules() {
        let tool = make_tool(true);
        let rules: Vec<RuleDefinition> = vec![];

        let content = RuleTranslator::translate(&tool, &rules);
        assert!(content.is_empty());
    }

    #[test]
    fn test_mandatory_rules_first() {
        let tool = make_tool(true);
        let rules = vec![
            make_rule("suggested", Severity::Suggestion),
            make_rule("required", Severity::Mandatory),
        ];

        let content = RuleTranslator::translate(&tool, &rules);
        let text = content.instructions.unwrap();

        // Required should come before suggested
        let req_pos = text.find("required").unwrap();
        let sug_pos = text.find("suggested").unwrap();
        assert!(req_pos < sug_pos);
    }

    #[test]
    fn test_includes_severity_markers() {
        let tool = make_tool(true);
        let rules = vec![
            make_rule("required", Severity::Mandatory),
            make_rule("suggested", Severity::Suggestion),
        ];

        let content = RuleTranslator::translate(&tool, &rules);
        let text = content.instructions.unwrap();

        assert!(text.contains("**[REQUIRED]**"));
        assert!(text.contains("[Suggested]"));
    }

    #[test]
    fn test_includes_examples() {
        let tool = make_tool(true);
        let mut rule = make_rule("with-examples", Severity::Mandatory);
        rule.examples = Some(RuleExamples {
            positive: vec!["good code".into()],
            negative: vec!["bad code".into()],
        });

        let content = RuleTranslator::translate(&tool, &[rule]);
        let text = content.instructions.unwrap();

        assert!(text.contains("**Good:**"));
        assert!(text.contains("good code"));
        assert!(text.contains("**Bad:**"));
        assert!(text.contains("bad code"));
    }

    #[test]
    fn test_includes_file_patterns() {
        let tool = make_tool(true);
        let mut rule = make_rule("with-targets", Severity::Mandatory);
        rule.targets = Some(RuleTargets {
            file_patterns: vec!["*.rs".into(), "*.ts".into()],
        });

        let content = RuleTranslator::translate(&tool, &[rule]);
        let text = content.instructions.unwrap();

        assert!(text.contains("**Applies to:**"));
        assert!(text.contains("*.rs"));
        assert!(text.contains("*.ts"));
    }
}
