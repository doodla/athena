//! Redaction of agent free text before it is recorded (design §5.6, M22).
//!
//! Three substitutions apply: every gitleaks finding becomes `[secret:<rule>]`, every email address
//! outside the allow-list becomes `[email]`, and machine-local paths the caller names (such as the home
//! directory, which carries a username) become placeholders. Redaction fails closed: without gitleaks,
//! nothing is recorded.

use crate::Result;
use crate::content::redact_emails;
use crate::secrets::Gitleaks;
use athena_data::Config;
use athena_data::config::PersonalData;

/// Redacts text with one configuration.
#[derive(Debug)]
pub struct Redactor {
    allow: PersonalData,
    gitleaks: Gitleaks,
    /// Machine-local paths and their placeholders, longest first.
    paths: Vec<(String, String)>,
}

impl Redactor {
    pub fn new(config: &Config) -> Result<Redactor> {
        Ok(Redactor {
            allow: config.personal_data.clone(),
            gitleaks: Gitleaks::new(config)?,
            paths: Vec::new(),
        })
    }

    /// Replaces `path` with `placeholder` wherever it appears. Longer paths are replaced first, so a
    /// directory inside another keeps its own placeholder.
    pub fn with_path(mut self, path: &std::path::Path, placeholder: &str) -> Redactor {
        let text = path.display().to_string();
        if text.len() > 1 {
            self.paths.push((
                text.trim_end_matches('/').to_string(),
                placeholder.to_string(),
            ));
            self.paths.sort_by_key(|(p, _)| std::cmp::Reverse(p.len()));
        }
        self
    }

    /// Redacts one text.
    pub fn redact(&self, text: &str) -> Result<String> {
        let mut texts = vec![text.to_string()];
        self.redact_all(&mut texts)?;
        Ok(texts.pop().unwrap_or_default())
    }

    /// Redacts many texts with a single scan.
    pub fn redact_all(&self, texts: &mut [String]) -> Result<()> {
        if texts.iter().all(|t| t.trim().is_empty()) {
            return Ok(());
        }
        let joined = texts.join("\n");
        let mut findings = self.gitleaks.scan_text(&joined)?;
        // Replace longer secrets first, so a secret that contains another is replaced whole.
        findings.sort_by_key(|f| std::cmp::Reverse(f.secret.len()));
        for text in texts.iter_mut() {
            for finding in &findings {
                if !finding.secret.is_empty() && text.contains(&finding.secret) {
                    *text = text.replace(&finding.secret, &format!("[secret:{}]", finding.rule));
                }
            }
            *text = redact_emails(text, &self.allow);
            for (path, placeholder) in &self.paths {
                if text.contains(path.as_str()) {
                    *text = text.replace(path.as_str(), placeholder);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;

    #[test]
    fn tokens_and_private_addresses_are_redacted() {
        // M22 (blocks): an email address or generated token in a producer summary is redacted.
        let redactor = Redactor::new(&crate::builtin_config()).unwrap();
        let summary = format!(
            "Set auth to {} and mailed {} about it.",
            fake_token(),
            private_email()
        );
        let out = redactor.redact(&summary).unwrap();
        assert!(!out.contains(&fake_token()));
        assert!(!out.contains(&private_email()));
        assert!(out.contains("[secret:github-pat]"));
        assert!(out.contains("[email]"));
    }

    #[test]
    fn machine_paths_become_placeholders() {
        let redactor = Redactor::new(&crate::builtin_config())
            .unwrap()
            .with_path(std::path::Path::new("/home/jane"), "~")
            .with_path(
                std::path::Path::new("/home/jane/.local/state/athena/repo/jobs/j"),
                "<job>",
            );
        let out = redactor.redact("error at /home/jane/.local/state/athena/repo/jobs/j/verify/src/lib.rs and /home/jane/x").unwrap();
        assert_eq!(out, "error at <job>/verify/src/lib.rs and ~/x");
    }

    #[test]
    fn ordinary_text_is_unchanged() {
        // M22 (allows): ordinary text and noreply@ addresses.
        let redactor = Redactor::new(&crate::builtin_config()).unwrap();
        let mut texts = vec![
            "Added CHANGELOG.md.".to_string(),
            "Co-Authored-By: noreply@anthropic.com".to_string(),
        ];
        let before = texts.clone();
        redactor.redact_all(&mut texts).unwrap();
        assert_eq!(texts, before);
    }
}
