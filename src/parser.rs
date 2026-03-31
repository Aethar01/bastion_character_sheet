use crate::model::Ability;

fn parse_rich_spans(text: &str) -> Vec<crate::model::TextSpan> {
    let mut spans = Vec::new();
    let mut current_text = String::new();
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '*' {
            let mut inside = String::new();
            while let Some(nc) = chars.peek() {
                if *nc == '*' {
                    break;
                }
                inside.push(chars.next().unwrap());
            }
            if chars.peek() == Some(&'*') {
                chars.next();
                if !current_text.is_empty() {
                    spans.push(crate::model::TextSpan {
                        content: std::mem::take(&mut current_text),
                        bold: false,
                        italic: false,
                    });
                }
                if !inside.is_empty() {
                    spans.push(crate::model::TextSpan {
                        content: inside,
                        bold: true,
                        italic: false,
                    });
                }
            } else {
                current_text.push('*');
                current_text.push_str(&inside);
            }
        } else if c == '_' {
            let mut inside = String::new();
            while let Some(nc) = chars.peek() {
                if *nc == '_' {
                    break;
                }
                inside.push(chars.next().unwrap());
            }
            if chars.peek() == Some(&'_') {
                chars.next();
                if !current_text.is_empty() {
                    spans.push(crate::model::TextSpan {
                        content: std::mem::take(&mut current_text),
                        bold: false,
                        italic: false,
                    });
                }
                if !inside.is_empty() {
                    spans.push(crate::model::TextSpan {
                        content: inside,
                        bold: false,
                        italic: true,
                    });
                }
            } else {
                current_text.push('_');
                current_text.push_str(&inside);
            }
        } else {
            current_text.push(c);
        }
    }

    if !current_text.is_empty() {
        spans.push(crate::model::TextSpan {
            content: current_text,
            bold: false,
            italic: false,
        });
    }

    spans
}

pub fn process_text(text: &str) -> Vec<crate::model::TextSpan> {
    let mut result = String::new();
    let mut prev_was_newline = false;
    let mut prev_was_bullet = false;

    for line in text.lines() {
        let trimmed = line.trim();
        let is_bullet = trimmed.starts_with("- ");

        if trimmed.is_empty() {
            if !result.is_empty() {
                result.push('\n');
            }
            prev_was_newline = true;
            prev_was_bullet = false;
        } else if is_bullet {
            if !result.is_empty() && !prev_was_newline {
                result.push('\n');
            } else if prev_was_bullet {
                result.push('\n');
            }
            let bullet_content = &trimmed[2..];
            result.push_str("• ");
            result.push_str(bullet_content);
            prev_was_newline = true;
            prev_was_bullet = true;
        } else {
            if !result.is_empty() {
                if prev_was_newline {
                    result.push('\n');
                } else if prev_was_bullet {
                    result.push('\n');
                } else {
                    result.push(' ');
                }
            }
            result.push_str(trimmed);
            prev_was_newline = false;
            prev_was_bullet = false;
        }
    }

    parse_rich_spans(&result)
}

pub fn parse_bastion_abilities(content: &str) -> Vec<Ability> {
    let mut abilities = Vec::new();

    let mut remaining = content;

    while let Some(start_idx) = remaining.find("title: \"") {
        let title_start = start_idx + 8; // length of "title: \""
        remaining = &remaining[title_start..];

        let title_end = match remaining.find("\"") {
            Some(idx) => idx,
            None => break,
        };

        let mut title = remaining[..title_end].to_string();
        title = title.replace("\\n", " ").replace("\n", " ");
        remaining = &remaining[title_end + 1..];

        // Find tags
        let tags_start_keyword = match remaining.find("tags:") {
            Some(idx) => idx,
            None => break,
        };
        remaining = &remaining[tags_start_keyword..];

        let tags_open = match remaining.find("(") {
            Some(idx) => idx + 1,
            None => break,
        };
        remaining = &remaining[tags_open..];

        let tags_close = match remaining.find(")") {
            Some(idx) => idx,
            None => break,
        };

        let tags_str = &remaining[..tags_close];
        let mut tags_list = Vec::new();
        let mut t_rem = tags_str;
        while let Some(q_start) = t_rem.find("\"") {
            t_rem = &t_rem[q_start + 1..];
            if let Some(q_end) = t_rem.find("\"") {
                tags_list.push(t_rem[..q_end].to_string());
                t_rem = &t_rem[q_end + 1..];
            } else {
                break;
            }
        }

        remaining = &remaining[tags_close + 1..];

        // Find desc
        let desc_start_keyword = match remaining.find("desc:") {
            Some(idx) => idx,
            None => break,
        };
        remaining = &remaining[desc_start_keyword..];

        let desc_open = match remaining.find("[") {
            Some(idx) => idx + 1,
            None => break,
        };
        remaining = &remaining[desc_open..];

        let mut nest = 1;
        let mut desc_end = 0;
        for (i, c) in remaining.char_indices() {
            if c == '[' {
                nest += 1;
            } else if c == ']' {
                nest -= 1;
                if nest == 0 {
                    desc_end = i;
                    break;
                }
            }
        }

        let desc_text = remaining[..desc_end].trim().to_string();
        remaining = &remaining[desc_end + 1..];

        // Find body
        let body_start_keyword = match remaining.find("body:") {
            Some(idx) => idx,
            None => break,
        };
        remaining = &remaining[body_start_keyword..];

        let body_open = match remaining.find("[") {
            Some(idx) => idx + 1,
            None => break,
        };
        remaining = &remaining[body_open..];

        nest = 1;
        let mut body_end = 0;
        for (i, c) in remaining.char_indices() {
            if c == '[' {
                nest += 1;
            } else if c == ']' {
                nest -= 1;
                if nest == 0 {
                    body_end = i;
                    break;
                }
            }
        }

        let body_text = remaining[..body_end].trim().to_string();
        remaining = &remaining[body_end + 1..];

        let joined_tags = tags_list.join(", ");

        let desc_text = desc_text.replace("#sym.times", "x");

        let body_text = body_text.replace("#sym.times", "x");

        let desc_spans = process_text(&desc_text);
        let body_spans = process_text(&body_text);

        abilities.push(Ability {
            name: title.trim().to_string(),
            tags: joined_tags,
            body: body_text.trim().to_string(),
            desc: desc_text.trim().to_string(),
            body_spans: body_spans,
            desc_spans: desc_spans,
            prepared: false,
        });
    }

    abilities
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bastion() {
        let content = r#"
#let cards = (
  (
    title: "Inspirational\nPerformance",
    tags: ("Maneuver", "2 Actions",),
    desc: [
      You bolster your Allies spirits.
    ],
    body: [
      All Allies gain +1d6.
    ]
  ),
  (
    title: "Tricks of the Trade",
    tags: ("Passive",),
    desc: [
      Over your travels you’ve learned how to say a few words.
    ],
    body: [
      When you prepare this ability pick Spells or Miracles.
    ]
  )
)
        "#;

        let parsed = parse_bastion_abilities(content);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].name, "Inspirational Performance");
        assert_eq!(parsed[0].tags, "Maneuver, 2 Actions");
        assert_eq!(parsed[0].body, "All Allies gain +1d6.");
        assert_eq!(parsed[0].desc, "You bolster your Allies spirits.");

        assert_eq!(parsed[1].name, "Tricks of the Trade");
        assert_eq!(parsed[1].tags, "Passive");
        assert_eq!(
            parsed[1].body,
            "When you prepare this ability pick Spells or Miracles."
        );
        assert_eq!(
            parsed[1].desc,
            "Over your travels you’ve learned how to say a few words."
        );
    }
}
