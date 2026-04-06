use crate::models::{ChecklistItem, ChecklistMeta, RunMetadata, Section};
use serde::Serialize;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ParseMode {
    Checklist,
    Run,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RunAnnotation {
    Note,
}

fn split_frontmatter(content: &str) -> Option<(&str, &str)> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }
    let after_first = &trimmed[3..];
    let end = after_first.find("\n---")?;
    let frontmatter = after_first[..end].trim();
    let body = after_first[end + 4..].trim_start_matches(['\r', '\n']);
    Some((frontmatter, body))
}

pub fn parse_checklist(content: &str) -> Result<(ChecklistMeta, Vec<Section>), String> {
    let (fm, body) =
        split_frontmatter(content).ok_or_else(|| "Missing YAML frontmatter".to_string())?;

    let meta: ChecklistMeta =
        serde_yaml::from_str(fm).map_err(|e| format!("Invalid frontmatter: {e}"))?;

    let sections = parse_body(body, ParseMode::Checklist);
    Ok((meta, sections))
}

pub fn parse_meta(content: &str) -> Result<ChecklistMeta, String> {
    let (fm, _) =
        split_frontmatter(content).ok_or_else(|| "Missing YAML frontmatter".to_string())?;

    serde_yaml::from_str(fm).map_err(|e| format!("Invalid frontmatter: {e}"))
}

pub fn parse_run(content: &str) -> Result<(RunMetadata, Vec<Section>), String> {
    let (fm, body) =
        split_frontmatter(content).ok_or_else(|| "Missing YAML frontmatter".to_string())?;

    let meta: RunMetadata =
        serde_yaml::from_str(fm).map_err(|e| format!("Invalid frontmatter: {e}"))?;

    let sections = parse_body(body, ParseMode::Run);
    Ok((meta, sections))
}

fn parse_body(body: &str, mode: ParseMode) -> Vec<Section> {
    let mut sections: Vec<Section> = Vec::new();
    let mut current_heading = String::new();
    let mut current_items: Vec<ChecklistItem> = Vec::new();
    let mut in_code_block = false;
    let mut code_lines: Vec<String> = Vec::new();
    let mut code_lang: Option<String> = None;
    let mut active_run_annotation: Option<RunAnnotation> = None;

    for line in body.lines() {
        if in_code_block {
            let trimmed = line.trim();
            if trimmed == "```" {
                if let Some(last) = current_items.last_mut() {
                    last.code = Some(code_lines.join("\n"));
                    last.code_lang = code_lang.take();
                }
                code_lines.clear();
                in_code_block = false;
            } else {
                let stripped = if let Some(s) = line.strip_prefix("  ") {
                    s
                } else {
                    line
                };
                code_lines.push(stripped.to_string());
            }
        } else if let Some(heading) = line.strip_prefix("## ") {
            active_run_annotation = None;
            if !current_items.is_empty() || !sections.is_empty() || !current_heading.is_empty() {
                sections.push(Section {
                    heading: current_heading.clone(),
                    items: std::mem::take(&mut current_items),
                });
            }
            current_heading = heading.trim().to_string();
        } else if let Some(item) = parse_checkbox_line(line) {
            active_run_annotation = None;
            current_items.push(item);
        } else if !line.trim().is_empty() {
            let trimmed = line.trim();
            if mode == ParseMode::Run {
                if let Some(note) = trimmed
                    .strip_prefix("> Note: ")
                    .or_else(|| trimmed.strip_prefix("> Note:"))
                {
                    if let Some(last) = current_items.last_mut() {
                        let note = note.trim().to_string();
                        last.note = if note.is_empty() { None } else { Some(note) };
                    }
                    active_run_annotation = Some(RunAnnotation::Note);
                    continue;
                }
                if let Some(reason) = trimmed
                    .strip_prefix("> Skipped: ")
                    .or_else(|| trimmed.strip_prefix("> Skipped:"))
                {
                    if let Some(last) = current_items.last_mut() {
                        let r = reason.trim().to_string();
                        last.skip_reason = if r.is_empty() { None } else { Some(r) };
                    }
                    active_run_annotation = None;
                    continue;
                }
                if active_run_annotation == Some(RunAnnotation::Note) {
                    if trimmed == ">" {
                        append_note_line(&mut current_items, "");
                        continue;
                    }
                    if let Some(note_line) = trimmed
                        .strip_prefix("> ")
                        .or_else(|| trimmed.strip_prefix(">"))
                    {
                        append_note_line(&mut current_items, note_line);
                        continue;
                    }
                }
            }

            active_run_annotation = None;
            if let Some(after_fence) = trimmed.strip_prefix("```") {
                in_code_block = true;
                let lang = after_fence.trim().to_string();
                code_lang = if lang.is_empty() { None } else { Some(lang) };
                code_lines.clear();
            } else if let Some(last) = current_items.last_mut() {
                if last.description.is_none() {
                    let desc = trimmed.to_string();
                    if !desc.is_empty() {
                        last.description = Some(desc);
                    }
                }
            }
        } else {
            active_run_annotation = None;
        }
    }

    if !current_items.is_empty() || !current_heading.is_empty() {
        sections.push(Section {
            heading: current_heading,
            items: current_items,
        });
    }

    sections
}

fn append_note_line(current_items: &mut [ChecklistItem], line: &str) {
    let Some(last) = current_items.last_mut() else {
        return;
    };

    let note = last.note.get_or_insert_with(String::new);
    note.push('\n');
    note.push_str(line);
}

fn parse_checkbox_line(line: &str) -> Option<ChecklistItem> {
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed
        .strip_prefix("- [x] ")
        .or_else(|| trimmed.strip_prefix("- [X] "))
    {
        Some(ChecklistItem {
            title: rest.trim().to_string(),
            description: None,
            code: None,
            code_lang: None,
            note: None,
            skip_reason: None,
            checked: true,
        })
    } else {
        trimmed.strip_prefix("- [ ] ").map(|rest| ChecklistItem {
            title: rest.trim().to_string(),
            description: None,
            code: None,
            code_lang: None,
            note: None,
            skip_reason: None,
            checked: false,
        })
    }
}

pub fn serialize_checklist(meta: &ChecklistMeta, sections: &[Section]) -> String {
    let mut out = String::new();

    let frontmatter = ChecklistFrontmatter {
        title: &meta.title,
        description: meta.description.as_deref(),
        tags: if meta.tags.is_empty() {
            None
        } else {
            Some(meta.tags.as_slice())
        },
    };
    out.push_str(&serialize_frontmatter(&frontmatter));

    for section in sections {
        out.push('\n');
        if !section.heading.is_empty() {
            out.push_str(&format!("## {}\n", section.heading));
        }
        for item in &section.items {
            let checkbox = if item.checked { "[x]" } else { "[ ]" };
            out.push_str(&format!("- {} {}\n", checkbox, item.title));
            if let Some(desc) = &item.description {
                out.push_str(&format!("  {}\n", desc));
            }
            if let Some(code) = &item.code {
                let lang = item.code_lang.as_deref().unwrap_or("");
                out.push_str(&format!("  ```{}\n", lang));
                for code_line in code.lines() {
                    out.push_str(&format!("  {}\n", code_line));
                }
                out.push_str("  ```\n");
            }
        }
    }

    out
}
pub fn serialize_run(run_meta: &RunMetadata, sections: &[Section]) -> String {
    let mut out = String::new();

    let frontmatter = RunFrontmatter {
        template: &run_meta.template,
        title: &run_meta.title,
        date: &run_meta.date,
        notes: run_meta.notes.as_deref(),
    };
    out.push_str(&serialize_frontmatter(&frontmatter));

    for section in sections {
        out.push('\n');
        if !section.heading.is_empty() {
            out.push_str(&format!("## {}\n", section.heading));
        }
        for item in &section.items {
            let checkbox = if item.checked { "[x]" } else { "[ ]" };
            out.push_str(&format!("- {} {}\n", checkbox, item.title));
            if let Some(desc) = &item.description {
                out.push_str(&format!("  {}\n", desc));
            }
            if let Some(code) = &item.code {
                let lang = item.code_lang.as_deref().unwrap_or("");
                out.push_str(&format!("  ```{}\n", lang));
                for code_line in code.lines() {
                    out.push_str(&format!("  {}\n", code_line));
                }
                out.push_str("  ```\n");
            }
            if let Some(note) = &item.note {
                let mut lines = note.lines();
                if let Some(first_line) = lines.next() {
                    out.push_str(&format!("  > Note: {}\n", first_line));
                    for line in lines {
                        if line.is_empty() {
                            out.push_str("  >\n");
                        } else {
                            out.push_str(&format!("  > {}\n", line));
                        }
                    }
                }
            }
            if let Some(reason) = &item.skip_reason {
                out.push_str(&format!("  > Skipped: {}\n", reason));
            }
        }
    }

    out
}

#[derive(Serialize)]
struct ChecklistFrontmatter<'a> {
    title: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<&'a [String]>,
}

#[derive(Serialize)]
struct RunFrontmatter<'a> {
    template: &'a str,
    title: &'a str,
    date: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<&'a str>,
}

fn serialize_frontmatter<T: Serialize>(value: &T) -> String {
    let yaml = serde_yaml::to_string(value).expect("frontmatter serialization should succeed");
    format!("---\n{yaml}---\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_checklist() {
        let input = r#"---
title: Test Checklist
description: A test
tags: [foo, bar]
---

## Section One
- [ ] First item
  Description of first item
- [x] Second item

## Section Two
- [ ] Third item
"#;
        let (meta, sections) = parse_checklist(input).unwrap();
        assert_eq!(meta.title, "Test Checklist");
        assert_eq!(meta.tags, vec!["foo", "bar"]);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].heading, "Section One");
        assert_eq!(sections[0].items.len(), 2);
        assert_eq!(sections[0].items[0].title, "First item");
        assert_eq!(
            sections[0].items[0].description.as_deref(),
            Some("Description of first item")
        );
        assert!(!sections[0].items[0].checked);
        assert!(sections[0].items[1].checked);
        assert_eq!(sections[1].items.len(), 1);
    }

    #[test]
    fn test_parse_code_snippet() {
        let input = r#"---
title: Code Test
---

## Setup
- [ ] Install dependencies
  Run this in the project root
  ```bash
  npm install
  npm run build
  ```
- [ ] Plain item
"#;
        let (_, sections) = parse_checklist(input).unwrap();
        let item = &sections[0].items[0];
        assert_eq!(item.title, "Install dependencies");
        assert_eq!(
            item.description.as_deref(),
            Some("Run this in the project root")
        );
        assert_eq!(item.code.as_deref(), Some("npm install\nnpm run build"));
        assert_eq!(item.code_lang.as_deref(), Some("bash"));
        assert!(sections[0].items[1].code.is_none());
    }

    #[test]
    fn test_roundtrip() {
        let meta = ChecklistMeta {
            title: "Test".to_string(),
            description: Some("A test checklist".to_string()),
            tags: vec!["a".to_string(), "b".to_string()],
        };
        let sections = vec![Section {
            heading: "Main".to_string(),
            items: vec![
                ChecklistItem {
                    title: "Do thing".to_string(),
                    description: Some("How to do it".to_string()),
                    code: None,
                    code_lang: None,
                    note: None,
                    skip_reason: None,
                    checked: false,
                },
                ChecklistItem {
                    title: "Done thing".to_string(),
                    description: None,
                    code: None,
                    code_lang: None,
                    note: None,
                    skip_reason: None,
                    checked: true,
                },
            ],
        }];

        let serialized = serialize_checklist(&meta, &sections);
        let (parsed_meta, parsed_sections) = parse_checklist(&serialized).unwrap();
        assert_eq!(parsed_meta.title, "Test");
        assert_eq!(parsed_sections[0].items.len(), 2);
        assert_eq!(
            parsed_sections[0].items[0].description.as_deref(),
            Some("How to do it")
        );
    }

    #[test]
    fn test_roundtrip_with_code() {
        let meta = ChecklistMeta {
            title: "Code RT".to_string(),
            description: None,
            tags: vec![],
        };
        let sections = vec![Section {
            heading: "Setup".to_string(),
            items: vec![ChecklistItem {
                title: "Run setup".to_string(),
                description: Some("Execute the following".to_string()),
                code: Some("echo hello\necho world".to_string()),
                code_lang: Some("bash".to_string()),
                note: None,
                skip_reason: None,
                checked: false,
            }],
        }];

        let serialized = serialize_checklist(&meta, &sections);
        let (_, parsed_sections) = parse_checklist(&serialized).unwrap();
        let item = &parsed_sections[0].items[0];
        assert_eq!(item.code.as_deref(), Some("echo hello\necho world"));
        assert_eq!(item.code_lang.as_deref(), Some("bash"));
        assert_eq!(item.description.as_deref(), Some("Execute the following"));
    }

    #[test]
    fn test_skip_reason_roundtrip() {
        let run_meta = RunMetadata {
            template: "test".to_string(),
            title: "Test Run".to_string(),
            date: "2026-03-08 09:15".to_string(),
            notes: None,
        };
        let sections = vec![Section {
            heading: "Setup".to_string(),
            items: vec![
                ChecklistItem {
                    title: "Done item".to_string(),
                    description: None,
                    code: None,
                    code_lang: None,
                    note: Some("Finished during maintenance window\nVerified by QA".to_string()),
                    skip_reason: None,
                    checked: true,
                },
                ChecklistItem {
                    title: "Skipped item".to_string(),
                    description: Some("Some desc".to_string()),
                    code: None,
                    code_lang: None,
                    note: Some("Waiting on another team".to_string()),
                    skip_reason: Some("Not needed for this server".to_string()),
                    checked: false,
                },
            ],
        }];

        let serialized = serialize_run(&run_meta, &sections);
        assert!(serialized.contains("> Note: Finished during maintenance window"));
        assert!(serialized.contains("> Verified by QA"));
        assert!(serialized.contains("> Note: Waiting on another team"));
        assert!(serialized.contains("> Skipped: Not needed for this server"));

        let (_, parsed_sections) = parse_run(&serialized).unwrap();
        assert_eq!(
            parsed_sections[0].items[0].note.as_deref(),
            Some("Finished during maintenance window\nVerified by QA")
        );
        assert!(parsed_sections[0].items[0].skip_reason.is_none());
        assert_eq!(
            parsed_sections[0].items[1].note.as_deref(),
            Some("Waiting on another team")
        );
        assert_eq!(
            parsed_sections[0].items[1].skip_reason.as_deref(),
            Some("Not needed for this server")
        );
    }

    #[test]
    fn test_checklist_ignores_run_note_annotations() {
        let input = r#"---
title: Checklist
---

## Main
- [ ] Item
  > Note: Should not become checklist data
"#;

        let (_, sections) = parse_checklist(input).unwrap();
        assert!(sections[0].items[0].note.is_none());
        assert_eq!(
            sections[0].items[0].description.as_deref(),
            Some("> Note: Should not become checklist data")
        );
    }

    #[test]
    fn test_multiline_note_roundtrip() {
        let run_meta = RunMetadata {
            template: "deploy".to_string(),
            title: "Multiline Note".to_string(),
            date: "2026-04-06 14:30".to_string(),
            notes: None,
        };
        let sections = vec![Section {
            heading: "Checks".to_string(),
            items: vec![ChecklistItem {
                title: "Review output".to_string(),
                description: None,
                code: None,
                code_lang: None,
                note: Some("First line\nSecond line\n\nFourth line".to_string()),
                skip_reason: None,
                checked: false,
            }],
        }];

        let serialized = serialize_run(&run_meta, &sections);
        assert!(serialized.contains("> Note: First line"));
        assert!(serialized.contains("> Second line"));
        assert!(serialized.contains("  >\n"));
        assert!(serialized.contains("> Fourth line"));

        let (_, parsed_sections) = parse_run(&serialized).unwrap();
        assert_eq!(
            parsed_sections[0].items[0].note.as_deref(),
            Some("First line\nSecond line\n\nFourth line")
        );
    }

    #[test]
    fn test_special_characters_roundtrip() {
        let meta = ChecklistMeta {
            title: "Deploy: \"blue\"\nnow".to_string(),
            description: Some("Comma, colon: quote \" and newline\nkept".to_string()),
            tags: vec!["ops,prod".to_string(), "needs:review".to_string()],
        };
        let sections = vec![Section {
            heading: "Main".to_string(),
            items: vec![ChecklistItem {
                title: "Do thing".to_string(),
                description: None,
                code: None,
                code_lang: None,
                note: None,
                skip_reason: None,
                checked: false,
            }],
        }];

        let serialized = serialize_checklist(&meta, &sections);
        let (parsed_meta, _) = parse_checklist(&serialized).unwrap();

        assert_eq!(parsed_meta.title, meta.title);
        assert_eq!(parsed_meta.description, meta.description);
        assert_eq!(parsed_meta.tags, meta.tags);

        let run_meta = RunMetadata {
            template: "deploy".to_string(),
            title: "Run \"1\"".to_string(),
            date: "2026-04-06 14:30".to_string(),
            notes: Some("Line 1\nLine 2: done".to_string()),
        };
        let serialized_run = serialize_run(&run_meta, &sections);
        let (parsed_run_meta, _) = parse_run(&serialized_run).unwrap();

        assert_eq!(parsed_run_meta.title, run_meta.title);
        assert_eq!(parsed_run_meta.notes, run_meta.notes);
    }
}
