use crate::installer::SkillEntry;
use crate::registry::InstallSecurityCheck;
use crate::ui::{bold, brand_cyan, dim, green, magenta, yellow};

use super::helpers::{format_skill_label, truncate_visible, visible_pad, wrap_text};
use super::types::{DisplayCombo, DisplayTechnology};

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
            brand_cyan("   ◆ ") + &bold("Tecnologías detectadas:")
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
            brand_cyan("   ◆ ")
                + &bold("Frontend web detectado ")
                + &dim("(a partir de archivos del proyecto)")
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
        brand_cyan("   ◆ ") + &bold("Skills por instalar ") + &dim(&count_label)
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
    out.push_str(&format!(
        "{}\n",
        brand_cyan("   ◆ ") + &bold("Verificaciones de seguridad")
    ));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::strip_ansi;

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
        let tech_rows: Vec<&str> = plain.lines().filter(|l| l.contains("Tech")).collect();
        assert_eq!(
            tech_rows.len(),
            3,
            "expected 3 rows, got {tech_rows:?} in {}",
            plain
        );
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
        let alpha_pos = plain.find("alpha-skill").unwrap();
        let zebra_pos = plain.find("zebra-skill").unwrap();
        assert!(alpha_pos < zebra_pos);
        assert!(plain.contains("Skill"));
        assert!(plain.contains("Hallazgos"));
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
        assert!(plain.contains('…'));
    }
}
