use crate::installer::SkillEntry;
use crate::registry::InstallSecurityCheck;
use crate::ui::{bold, cyan, dim, green, magenta, strip_ansi, yellow};

// ── Types for display ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DisplayTechnology {
    pub id: String,
    pub name: String,
    pub skills: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DisplayCombo {
    pub id: String,
    pub name: String,
}

// ── Helpers ────────────────────────────────────────────────────────

pub fn format_skill_label(skill: &str, styled: bool) -> String {
    if skill.to_lowercase().starts_with("http://") || skill.to_lowercase().starts_with("https://") {
        if styled {
            return cyan(skill);
        } else {
            return skill.to_string();
        }
    }
    let parts: Vec<&str> = skill.split('/').collect();
    if parts.len() != 3 {
        if styled {
            return cyan(skill);
        } else {
            return skill.to_string();
        }
    }
    let author = parts[0];
    let skill_name = parts[2];
    if !styled {
        return format!("{author} › {skill_name}");
    }
    // styled: muted(author) + gray("›") + cyan(bold(skillName))
    format!(
        "{} {} {}",
        crate::ui::muted(author),
        crate::ui::gray("›"),
        cyan(&bold(skill_name))
    )
}

pub fn strip_ansi_owned(s: &str) -> String {
    strip_ansi(s)
}

pub fn visible_pad(value: &str, width: usize) -> String {
    let visible_len = strip_ansi(value).chars().count();
    if visible_len >= width {
        value.to_string()
    } else {
        format!("{}{}", value, " ".repeat(width - visible_len))
    }
}

pub fn truncate_visible(value: &str, width: usize) -> String {
    let plain = strip_ansi(value);
    let len = plain.chars().count();
    if len <= width {
        return value.to_string();
    }
    if width <= 1 {
        return "…".to_string();
    }
    let truncated: String = plain.chars().take(width - 1).collect();
    format!("{truncated}…")
}

pub fn wrap_text(value: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![value.to_string()];
    }
    let words: Vec<&str> = value.split_whitespace().collect();
    if words.is_empty() {
        return vec![String::new()];
    }
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    for word in words {
        if word.chars().count() > width {
            if !line.is_empty() {
                lines.push(line);
                line = String::new();
            }
            // split long word into width chunks
            let chars: Vec<char> = word.chars().collect();
            for chunk in chars.chunks(width) {
                lines.push(chunk.iter().collect());
            }
            continue;
        }
        let next = if line.is_empty() {
            word.to_string()
        } else {
            format!("{line} {word}")
        };
        if next.chars().count() > width {
            lines.push(line);
            line = word.to_string();
        } else {
            line = next;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

// ── Rendering ──────────────────────────────────────────────────────

pub fn format_detected(
    detected: &[DisplayTechnology],
    combos: &[DisplayCombo],
    is_frontend: bool,
) -> String {
    let mut out = String::new();
    if !detected.is_empty() {
        let with_skills: Vec<&DisplayTechnology> =
            detected.iter().filter(|t| !t.skills.is_empty()).collect();
        let without_skills: Vec<&DisplayTechnology> =
            detected.iter().filter(|t| t.skills.is_empty()).collect();
        let mut all_tech: Vec<&DisplayTechnology> = Vec::new();
        all_tech.extend(with_skills);
        all_tech.extend(without_skills);

        out.push_str(&format!(
            "{}\n",
            cyan("   ◆ ") + &bold("Tecnologías detectadas:")
        ));
        out.push('\n');

        const COLS: usize = 3;
        let max_len = all_tech
            .iter()
            .map(|t| t.name.chars().count())
            .max()
            .unwrap_or(0);
        let col_width = max_len + 3;

        let format_tech = |tech: &DisplayTechnology| -> String {
            let has_skills = !tech.skills.is_empty();
            let icon = if has_skills { green("✔") } else { dim("●") };
            let padded = format!(
                "{}{}",
                tech.name,
                " ".repeat(col_width - tech.name.chars().count())
            );
            if has_skills {
                format!("{icon} {padded}")
            } else {
                format!("{icon} {}", dim(&padded))
            }
        };

        for chunk in all_tech.chunks(COLS) {
            let row = chunk
                .iter()
                .map(|t| format_tech(t))
                .collect::<Vec<_>>()
                .join("");
            out.push_str(&format!("     {row}\n"));
        }

        if !combos.is_empty() {
            out.push('\n');
            out.push_str(&format!(
                "{}\n",
                magenta("   ◆ ") + &bold("Combinaciones detectadas:")
            ));
            out.push('\n');
            for combo in combos {
                out.push_str(&format!("{}{}\n", magenta("     ⚡ "), combo.name));
            }
        }
        out.push('\n');
    }

    if is_frontend && detected.is_empty() {
        out.push_str(&format!(
            "{}\n",
            cyan("   ◆ ") + &bold("Frontend web detectado ") + &dim("(a partir de archivos del proyecto)")
        ));
        out.push('\n');
    }

    out
}

pub fn print_detected(detected: &[DisplayTechnology], combos: &[DisplayCombo], is_frontend: bool) {
    let s = format_detected(detected, combos, is_frontend);
    if !s.is_empty() {
        print!("{s}");
    }
}

pub fn format_skills_list(skills: &[SkillEntry]) -> String {
    const INSTALLED_TAG: &str = " (instalada)";
    let entries: Vec<(String, String, bool)> = skills
        .iter()
        .map(|s| {
            let label = format_skill_label(&s.skill, false);
            let styled = format_skill_label(&s.skill, true);
            (label, styled, s.installed)
        })
        .collect();

    let max_effective = entries
        .iter()
        .map(|(label, _, installed)| {
            label.chars().count()
                + if *installed {
                    INSTALLED_TAG.chars().count()
                } else {
                    0
                }
        })
        .max()
        .unwrap_or(0);

    let new_count = skills.iter().filter(|s| !s.installed).count();
    let installed_count = skills.len() - new_count;
    let count_label = if installed_count > 0 {
        format!("({}, {} ya instaladas)", skills.len(), installed_count)
    } else {
        format!("({})", skills.len())
    };

    let mut out = String::new();
    out.push_str(&format!(
        "{}\n",
        cyan("   ◆ ") + &bold("Skills por instalar ") + &dim(&count_label)
    ));
    out.push('\n');

    for (i, (label, styled_label, _installed)) in entries.iter().enumerate() {
        let skill = &skills[i];
        let tech_sources: Vec<&String> = skill
            .sources
            .iter()
            .filter(|s| !s.contains(" + "))
            .collect();
        let installed_tag = if skill.installed {
            dim(INSTALLED_TAG)
        } else {
            String::new()
        };
        let effective_len = label.chars().count()
            + if skill.installed {
                INSTALLED_TAG.chars().count()
            } else {
                0
            };
        let pad = " ".repeat(max_effective.saturating_sub(effective_len));
        let num = format!("{:2}", i + 1);
        let source_suffix = if tech_sources.is_empty() {
            String::new()
        } else {
            format!(
                "  {}",
                dim(&format!(
                    "← {}",
                    tech_sources
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            )
        };
        let num_part = dim(&format!("   {num}."));
        let label_part = format!(" {styled_label}");
        out.push_str(&format!(
            "{num_part}{label_part}{installed_tag}{pad}{source_suffix}\n"
        ));
    }
    out.push('\n');
    out
}

pub fn print_skills_list(skills: &[SkillEntry]) {
    let s = format_skills_list(skills);
    print!("{s}");
}

fn format_security_findings(check: &InstallSecurityCheck) -> Option<String> {
    let findings: Vec<String> = check
        .findings
        .iter()
        .map(|f| f.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if findings.is_empty() {
        return None;
    }
    let summary = check.summary.trim().to_string();
    let mut parts = Vec::new();
    if !summary.is_empty() {
        parts.push(summary);
    }
    parts.push(findings.join("; "));
    Some(parts.join(" "))
}

pub fn format_security_checks(checks: &[InstallSecurityCheck]) -> String {
    let mut with_findings: Vec<(&InstallSecurityCheck, String)> = Vec::new();
    for check in checks {
        if let Some(f) = format_security_findings(check) {
            with_findings.push((check, f));
        }
    }
    if with_findings.is_empty() {
        return String::new();
    }
    with_findings.sort_by(|a, b| a.0.name.cmp(&b.0.name));

    let skill_width = {
        let max = with_findings
            .iter()
            .map(|(c, _)| c.name.chars().count())
            .max()
            .unwrap_or(5);
        max.clamp(5, 34)
    };
    let check_width = 12usize;
    let terminal_width = crossterm::terminal::size()
        .map(|(w, _)| w as usize)
        .unwrap_or(100);
    let findings_width = std::cmp::max(
        40,
        terminal_width.saturating_sub(skill_width + check_width + 16),
    );

    let mut out = String::new();
    out.push('\n');
    out.push_str(&format!("{}\n", cyan("   ◆ ") + &bold("Verificaciones de seguridad")));
    out.push('\n');
    out.push_str(&format!(
        "{}\n",
        dim(&format!(
            "   | {} | {} | {} |",
            visible_pad("Skill", skill_width),
            visible_pad("Verificación", check_width),
            visible_pad("Hallazgos", findings_width)
        ))
    ));
    out.push_str(&format!(
        "{}\n",
        dim(&format!(
            "   | {} | {} | {} |",
            "-".repeat(skill_width),
            "-".repeat(check_width),
            "-".repeat(findings_width)
        ))
    ));

    for (check, findings) in with_findings {
        let status = if check.status == "warning" {
            yellow("advertencia")
        } else {
            green("ok")
        };
        let lines = wrap_text(&findings, findings_width);
        out.push_str(&format!(
            "   | {} | {} | {} |\n",
            visible_pad(&truncate_visible(&check.name, skill_width), skill_width),
            visible_pad(&status, check_width),
            visible_pad(&lines[0], findings_width)
        ));
        for line in lines.iter().skip(1) {
            out.push_str(&format!(
                "   | {} | {} | {} |\n",
                visible_pad("", skill_width),
                visible_pad("", check_width),
                visible_pad(line, findings_width)
            ));
        }
    }
    out
}

pub fn print_security_checks(checks: &[InstallSecurityCheck]) {
    let s = format_security_checks(checks);
    if !s.is_empty() {
        print!("{s}");
    }
}

pub fn format_time(ms: u64) -> String {
    if ms < 1000 {
        return format!("{ms}ms");
    }
    let s = ms as f64 / 1000.0;
    if s < 60.0 {
        return format!("{:.1}s", s);
    }
    let m = (s / 60.0).floor() as u64;
    let rem = (s % 60.0).round() as u64;
    format!("{m}m {rem}s")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::InstallSecurityCheck;

    #[test]
    fn format_skill_label_plain_url() {
        let s = format_skill_label("https://example.com/skill", false);
        assert_eq!(s, "https://example.com/skill");
    }

    #[test]
    fn format_skill_label_plain_three_parts() {
        let s = format_skill_label("owner/repo/my-skill", false);
        assert_eq!(s, "owner › my-skill");
    }

    #[test]
    fn format_skill_label_plain_not_three() {
        let s = format_skill_label("owner/repo", false);
        assert_eq!(s, "owner/repo");
    }

    #[test]
    fn format_skill_label_styled_contains_parts() {
        let s = format_skill_label("owner/repo/my-skill", true);
        // Should contain author and skill name, with ANSI stripped still contains them
        let plain = strip_ansi(&s);
        assert!(plain.contains("owner"));
        assert!(plain.contains("my-skill"));
        assert!(plain.contains("›"));
    }

    #[test]
    fn three_col_rows_7_techs() {
        let techs: Vec<DisplayTechnology> = (0..7)
            .map(|i| DisplayTechnology {
                id: format!("tech{i}"),
                name: format!("Tech{i}"),
                skills: if i % 2 == 0 { vec!["s".into()] } else { vec![] },
            })
            .collect();
        let out = format_detected(&techs, &[], false);
        let plain = strip_ansi(&out);
        // Count rows that contain Tech
        let tech_rows: Vec<&str> = plain.lines().filter(|l| l.contains("Tech")).collect();
        // Should be 3 rows: 3,3,1
        assert_eq!(
            tech_rows.len(),
            3,
            "expected 3 rows, got {tech_rows:?} in {}",
            plain
        );
        // First row should have 3 techs
        assert!(tech_rows[0].matches("Tech").count() == 3);
        assert!(tech_rows[1].matches("Tech").count() == 3);
        assert!(tech_rows[2].matches("Tech").count() == 1);
    }

    #[test]
    fn three_col_single_tech() {
        let techs = vec![DisplayTechnology {
            id: "a".into(),
            name: "React".into(),
            skills: vec!["s".into()],
        }];
        let out = format_detected(&techs, &[], false);
        let plain = strip_ansi(&out);
        let rows: Vec<&str> = plain.lines().filter(|l| l.contains("React")).collect();
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn wrap_text_basic() {
        let lines = wrap_text("hello world foo bar", 10);
        // "hello world" 11 chars >10 so should wrap
        assert!(lines.len() >= 2);
        for l in &lines {
            assert!(l.chars().count() <= 10);
        }
    }

    #[test]
    fn wrap_text_long_word_split() {
        let long = "a".repeat(50);
        let lines = wrap_text(&long, 10);
        assert!(lines.len() == 5);
        for l in &lines {
            assert!(l.chars().count() <= 10);
        }
    }

    #[test]
    fn visible_pad_pads_correctly() {
        let s = green("hi");
        let padded = visible_pad(&s, 5);
        assert_eq!(strip_ansi(&padded).chars().count(), 5);
    }

    #[test]
    fn truncate_visible_truncates() {
        let s = "hello world";
        let t = truncate_visible(s, 5);
        assert_eq!(t.chars().count(), 5);
        assert!(t.ends_with('…'));
    }

    #[test]
    fn security_table_sorted_and_wrapped() {
        let checks = vec![
            InstallSecurityCheck {
                name: "zebra-skill".into(),
                status: "warning".into(),
                summary: "needs review".into(),
                findings: vec!["very long finding that should wrap across multiple lines because it is super long and exceeds width".into()],
            },
            InstallSecurityCheck {
                name: "alpha-skill".into(),
                status: "ok".into(),
                summary: "ok".into(),
                findings: vec!["short".into()],
            },
        ];
        let out = format_security_checks(&checks);
        let plain = strip_ansi(&out);
        // Should be sorted alpha before zebra
        let alpha_pos = plain.find("alpha-skill").unwrap();
        let zebra_pos = plain.find("zebra-skill").unwrap();
        assert!(alpha_pos < zebra_pos);
        // Should contain header
        assert!(plain.contains("Skill"));
        assert!(plain.contains("Hallazgos"));
        // long finding should be wrapped (multiple lines)
        assert!(plain.lines().count() > 5);
    }

    #[test]
    fn security_table_empty_returns_empty() {
        let out = format_security_checks(&[]);
        assert!(out.is_empty());
        let checks = vec![InstallSecurityCheck {
            name: "x".into(),
            status: "ok".into(),
            summary: "".into(),
            findings: vec![],
        }];
        let out2 = format_security_checks(&checks);
        assert!(out2.is_empty());
    }

    #[test]
    fn skill_34_truncate() {
        let long_name = "a".repeat(50);
        let checks = vec![InstallSecurityCheck {
            name: long_name.clone(),
            status: "warning".into(),
            summary: "s".into(),
            findings: vec!["f".into()],
        }];
        let out = format_security_checks(&checks);
        let plain = strip_ansi(&out);
        // Skill column width is min 34, so long name should be truncated to 34 chars with …
        // Check that plain contains truncated version (33 chars + …)
        assert!(plain.contains('…'));
    }

    #[test]
    fn format_time_cases() {
        assert_eq!(format_time(500), "500ms");
        assert_eq!(format_time(1500), "1.5s");
        assert_eq!(format_time(61_000), "1m 1s");
    }
}
