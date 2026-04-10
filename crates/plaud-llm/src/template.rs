//! Summarization prompt templates.
//!
//! Templates are Markdown files containing a system prompt for the LLM.
//! Five built-in templates are compiled into the binary; users can
//! override or extend them by placing `.md` files in
//! `~/.config/plaude/templates/`.

use std::path::{Path, PathBuf};

/// Number of built-in templates shipped with the binary.
const BUILTIN_COUNT: usize = 5;

/// Subdirectory within the config dir where user templates live.
const TEMPLATES_DIR: &str = "templates";

/// File extension for template files.
const TEMPLATE_EXT: &str = "md";

// ── Built-in template bodies ────────────────────────────────────────

const DEFAULT_BODY: &str = "\
You are a precise summarization assistant. Given the transcript of a \
voice recording, produce a clear, structured summary.

## Output format

- **Summary**: 2-4 sentence overview of the recording
- **Key Points**: Bulleted list of the most important topics discussed
- **Action Items**: Any tasks, deadlines, or follow-ups mentioned (if applicable)
- **Decisions**: Any decisions made during the recording (if applicable)

## Rules

- Be concise. Prefer short sentences over long ones.
- Preserve technical terms, proper nouns, and numbers exactly as spoken.
- If speakers are identified, attribute key points to speakers.
- Omit filler words, false starts, and repetitions.
- If a section has no content (e.g., no action items), omit it entirely.";

const MEETING_NOTES_BODY: &str = "\
You are a meeting notes assistant. Given the transcript of a meeting, \
produce structured meeting notes.

## Output format

- **Meeting Overview**: 1-2 sentence summary of the meeting purpose
- **Attendees**: List of participants (if identifiable from speaker labels)
- **Agenda Items**: Numbered list of topics discussed
- **Discussion**: Key points per agenda item, attributed to speakers when possible
- **Outcomes**: Decisions reached, consensus points
- **Next Steps**: Action items with owners and deadlines if mentioned

## Rules

- Use speaker labels when available.
- Keep each section concise — bullet points over paragraphs.
- If attendees cannot be identified, omit that section.
- Preserve any dates, numbers, or deadlines exactly as spoken.";

const ACTION_ITEMS_BODY: &str = "\
You are a task extraction assistant. Given the transcript of a \
recording, extract all action items, tasks, and commitments.

## Output format

Produce a numbered list. Each item must include:
- **Task**: What needs to be done
- **Owner**: Who is responsible (use speaker label or \"unassigned\")
- **Deadline**: When it is due (or \"no deadline mentioned\")
- **Priority**: High / Medium / Low (infer from context and urgency cues)

## Rules

- Only list concrete, actionable tasks — not general discussion topics.
- If no action items are found, respond with \"No action items identified.\"
- Preserve exact wording of commitments where possible.
- Include implicit tasks (e.g., \"I'll send that over\" → send document).";

const KEY_DECISIONS_BODY: &str = "\
You are a decision log assistant. Given the transcript of a recording, \
extract all decisions that were made.

## Output format

For each decision, include:
- **Decision**: What was decided
- **Context**: Why this decision was needed (1 sentence)
- **Alternatives**: Other options discussed, if any
- **Rationale**: Why this option was chosen
- **Owner**: Who is responsible for executing

## Rules

- Only include actual decisions, not open questions or future considerations.
- If no decisions were made, respond with \"No decisions identified.\"
- Distinguish between tentative agreements and firm decisions.
- Preserve the exact wording of decisions where possible.";

const BRIEF_BODY: &str = "\
You are a concise summarization assistant. Given the transcript of a \
voice recording, produce a brief executive summary.

## Rules

- Write exactly 2-3 sentences.
- Cover the main topic, key outcome, and any critical next step.
- No bullet points, no headers, no structure — just prose.
- Omit pleasantries, filler, and tangential discussion.
- If speakers are identified, mention the most relevant by name.";

/// Built-in template definitions: `(name, body)`.
const BUILTINS: [(&str, &str); BUILTIN_COUNT] = [
    ("default", DEFAULT_BODY),
    ("meeting-notes", MEETING_NOTES_BODY),
    ("action-items", ACTION_ITEMS_BODY),
    ("key-decisions", KEY_DECISIONS_BODY),
    ("brief", BRIEF_BODY),
];

// ── Public types ────────────────────────────────────────────────────

/// Where a template was loaded from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateSource {
    /// Compiled into the binary.
    BuiltIn,
    /// Loaded from user config directory.
    User,
}

impl std::fmt::Display for TemplateSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BuiltIn => f.write_str("built-in"),
            Self::User => f.write_str("user"),
        }
    }
}

/// A loaded summarization template.
#[derive(Debug, Clone)]
pub struct Template {
    /// Template name (filename stem without extension).
    pub name: String,
    /// The system prompt text sent to the LLM.
    pub body: String,
    /// Where this template was loaded from.
    pub source: TemplateSource,
}

/// Summary info for listing templates.
#[derive(Debug, Clone)]
pub struct TemplateInfo {
    /// Template name.
    pub name: String,
    /// Where this template was loaded from.
    pub source: TemplateSource,
    /// First non-empty line of the body, for preview.
    pub preview: String,
}

/// Registry of available templates (built-in + user).
#[derive(Debug)]
pub struct TemplateRegistry {
    templates: Vec<Template>,
}

impl TemplateRegistry {
    /// Load templates from built-ins and the user config directory.
    ///
    /// User templates in `<config_dir>/templates/*.md` override
    /// built-ins of the same name.
    pub fn load(config_dir: &Path) -> Self {
        let mut templates: Vec<Template> = BUILTINS
            .iter()
            .map(|(name, body)| Template {
                name: (*name).to_owned(),
                body: (*body).to_owned(),
                source: TemplateSource::BuiltIn,
            })
            .collect();

        let user_dir = config_dir.join(TEMPLATES_DIR);
        if let Ok(entries) = std::fs::read_dir(&user_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == TEMPLATE_EXT) {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if let Ok(body) = std::fs::read_to_string(&path) {
                            let name = stem.to_owned();
                            // Remove existing built-in of same name.
                            templates.retain(|t| t.name != name);
                            templates.push(Template {
                                name,
                                body,
                                source: TemplateSource::User,
                            });
                        }
                    }
                }
            }
        }

        Self { templates }
    }

    /// Get a template by name, or `None` if not found.
    pub fn get(&self, name: &str) -> Option<&Template> {
        self.templates.iter().find(|t| t.name == name)
    }

    /// List all available templates with summary info.
    pub fn list(&self) -> Vec<TemplateInfo> {
        let mut infos: Vec<TemplateInfo> = self
            .templates
            .iter()
            .map(|t| {
                let preview = first_nonempty_line(&t.body);
                TemplateInfo {
                    name: t.name.clone(),
                    source: t.source,
                    preview,
                }
            })
            .collect();
        infos.sort_by(|a, b| a.name.cmp(&b.name));
        infos
    }

    /// Get the body of a built-in template by name, regardless of
    /// user overrides. Used for `--export-template`.
    pub fn builtin_body(name: &str) -> Option<&'static str> {
        BUILTINS.iter().find(|(n, _)| *n == name).map(|(_, body)| *body)
    }

    /// List the names of all built-in templates.
    pub fn builtin_names() -> &'static [(&'static str, &'static str); BUILTIN_COUNT] {
        &BUILTINS
    }
}

/// Return the path to the user templates directory.
pub fn templates_dir(config_dir: &Path) -> PathBuf {
    config_dir.join(TEMPLATES_DIR)
}

/// Return the path where a user template file would live.
pub fn user_template_path(config_dir: &Path, name: &str) -> PathBuf {
    templates_dir(config_dir).join(format!("{name}.{TEMPLATE_EXT}"))
}

/// Return the first non-empty, non-whitespace-only line, truncated
/// to 60 chars for preview display.
fn first_nonempty_line(text: &str) -> String {
    const MAX_PREVIEW: usize = 60;
    let line = text.lines().find(|l| !l.trim().is_empty()).unwrap_or("").trim();
    if line.len() > MAX_PREVIEW {
        format!("{}...", &line[..MAX_PREVIEW])
    } else {
        line.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_count_is_five() {
        assert_eq!(BUILTINS.len(), BUILTIN_COUNT);
    }

    #[test]
    fn builtin_names() {
        let names: Vec<&str> = BUILTINS.iter().map(|(n, _)| *n).collect();
        assert_eq!(names, vec!["default", "meeting-notes", "action-items", "key-decisions", "brief"]);
    }

    #[test]
    fn builtins_are_nonempty() {
        for (name, body) in &BUILTINS {
            assert!(!body.is_empty(), "built-in template '{name}' has empty body");
        }
    }

    #[test]
    fn get_unknown_returns_none() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let reg = TemplateRegistry::load(tmp.path());
        assert!(reg.get("nonexistent").is_none());
    }

    #[test]
    fn get_builtin_returns_template() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let reg = TemplateRegistry::load(tmp.path());
        let t = reg.get("default").expect("default exists");
        assert_eq!(t.source, TemplateSource::BuiltIn);
        assert!(!t.body.is_empty());
    }

    #[test]
    fn user_overrides_builtin() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let tpl_dir = tmp.path().join("templates");
        std::fs::create_dir_all(&tpl_dir).expect("mkdir");
        std::fs::write(tpl_dir.join("default.md"), "My custom default prompt").expect("write");

        let reg = TemplateRegistry::load(tmp.path());
        let t = reg.get("default").expect("default exists");
        assert_eq!(t.source, TemplateSource::User);
        assert_eq!(t.body, "My custom default prompt");
    }

    #[test]
    fn user_template_added_to_list() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let tpl_dir = tmp.path().join("templates");
        std::fs::create_dir_all(&tpl_dir).expect("mkdir");
        std::fs::write(tpl_dir.join("my-custom.md"), "Custom prompt here").expect("write");

        let reg = TemplateRegistry::load(tmp.path());
        let t = reg.get("my-custom").expect("custom exists");
        assert_eq!(t.source, TemplateSource::User);

        let infos = reg.list();
        let names: Vec<&str> = infos.iter().map(|i| i.name.as_str()).collect();
        assert!(names.contains(&"my-custom"));
        // Built-ins still present.
        assert!(names.contains(&"default"));
    }

    #[test]
    fn list_includes_all_sources() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let tpl_dir = tmp.path().join("templates");
        std::fs::create_dir_all(&tpl_dir).expect("mkdir");
        std::fs::write(tpl_dir.join("custom.md"), "Extra template").expect("write");

        let reg = TemplateRegistry::load(tmp.path());
        let infos = reg.list();
        // 5 built-ins + 1 custom = 6.
        assert_eq!(infos.len(), BUILTIN_COUNT + 1);
    }

    #[test]
    fn list_sorted_by_name() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let reg = TemplateRegistry::load(tmp.path());
        let infos = reg.list();
        let names: Vec<&str> = infos.iter().map(|i| i.name.as_str()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }

    #[test]
    fn builtin_body_returns_content() {
        let body = TemplateRegistry::builtin_body("brief").expect("brief exists");
        assert!(body.contains("executive summary"));
    }

    #[test]
    fn builtin_body_unknown_returns_none() {
        assert!(TemplateRegistry::builtin_body("nonexistent").is_none());
    }

    #[test]
    fn preview_truncates_long_lines() {
        let long = "A".repeat(100);
        let preview = first_nonempty_line(&long);
        assert!(preview.len() <= 63 + 3); // 60 + "..."
        assert!(preview.ends_with("..."));
    }

    #[test]
    fn preview_skips_empty_lines() {
        let text = "\n\n  \nActual content here";
        let preview = first_nonempty_line(text);
        assert_eq!(preview, "Actual content here");
    }
}
