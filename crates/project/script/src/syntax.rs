//! Line-based reader for the script format.
//!
//! The format is plain Markdown that stays readable in any editor:
//!
//! ```text
//! ---
//! size: 1920x1080
//! fps: 30
//! wpm: 150
//! captions: true
//! ---
//!
//! # Headings only organise the file
//!
//! ![](clips/city.mp4){at=12 dur=4 in=fade}
//! ![](img/map.png){zoom=1..1.3}
//!
//! > Voice-over text. The beat lasts as long as it takes to read.
//! ```
//!
//! A beat is a run of media lines followed by the quote lines that narrate
//! them. The next media line, a heading or a `---` rule starts a new beat.

use std::collections::BTreeMap;

const BYTE_ORDER_MARK: char = '\u{feff}';

pub struct Script {
    pub settings: BTreeMap<String, String>,
    pub beats: Vec<Beat>,
}

#[derive(Default)]
pub struct Beat {
    /// 1-based line of the first entry, for error messages.
    pub line: usize,
    pub media: Vec<Media>,
    pub text: Vec<String>,
}

pub struct Media {
    pub line: usize,
    pub path: String,
    pub attributes: BTreeMap<String, String>,
}

impl Beat {
    fn is_empty(&self) -> bool {
        self.media.is_empty() && self.text.is_empty()
    }
}

pub fn parse(source: &str) -> Result<Script, String> {
    let source = source.trim_start_matches(BYTE_ORDER_MARK);
    let mut lines = source.lines().enumerate().peekable();
    let mut settings = BTreeMap::new();
    if lines.peek().is_some_and(|(_, line)| line.trim() == "---") {
        lines.next();
        loop {
            let (index, line) = lines
                .next()
                .ok_or("front matter starting at line 1 is never closed with ---")?;
            let line = line.trim();
            if line == "---" {
                break;
            }
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, value) = line
                .split_once(':')
                .ok_or_else(|| format!("line {}: expected `key: value`", index + 1))?;
            settings.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    let mut beats = Vec::new();
    let mut beat = Beat::default();
    let mut in_comment = false;
    for (index, raw) in lines {
        let number = index + 1;
        let line = raw.trim();
        // HTML comments hold director notes that never reach the video.
        if in_comment || line.starts_with("<!--") {
            in_comment = !line.contains("-->");
            continue;
        }
        if line.starts_with('#') || line == "---" {
            flush(&mut beats, &mut beat);
        } else if let Some(text) = line.strip_prefix('>') {
            if beat.is_empty() {
                beat.line = number;
            }
            let text = text.trim();
            if !text.is_empty() {
                beat.text.push(text.to_string());
            }
        } else if line.starts_with("![") {
            if !beat.text.is_empty() {
                flush(&mut beats, &mut beat);
            }
            if beat.is_empty() {
                beat.line = number;
            }
            beat.media.push(media(line, number)?);
        }
        // Any other prose stays in the file as notes for the author.
    }
    flush(&mut beats, &mut beat);
    Ok(Script { settings, beats })
}

fn flush(beats: &mut Vec<Beat>, beat: &mut Beat) {
    if !beat.is_empty() {
        beats.push(std::mem::take(beat));
    }
}

fn media(line: &str, number: usize) -> Result<Media, String> {
    let open = line
        .find("](")
        .ok_or_else(|| format!("line {number}: media link must look like ![](path)"))?;
    let rest = &line[open + 2..];
    // Attributes follow the closing parenthesis directly, so paths may contain parentheses.
    let (link, attributes) = match rest.rfind("){") {
        Some(close) => (&rest[..close], &rest[close + 1..]),
        None => {
            let close = rest
                .rfind(')')
                .ok_or_else(|| format!("line {number}: media link is missing `)`"))?;
            (&rest[..close], &rest[close + 1..])
        }
    };
    let path = strip_title(link.trim());
    if path.is_empty() {
        return Err(format!("line {number}: media link has no path"));
    }
    let attributes = attributes.trim();
    let mut parsed = BTreeMap::new();
    if !attributes.is_empty() {
        let body = attributes
            .strip_prefix('{')
            .and_then(|rest| rest.strip_suffix('}'))
            .ok_or_else(|| format!("line {number}: attributes must be wrapped in {{...}}"))?;
        for pair in body.split_whitespace() {
            let (key, value) = pair
                .split_once('=')
                .ok_or_else(|| format!("line {number}: attribute `{pair}` must be key=value"))?;
            parsed.insert(key.to_string(), value.to_string());
        }
    }
    Ok(Media {
        line: number,
        path: path.to_string(),
        attributes: parsed,
    })
}

/// Drops a Markdown link title: `clip.mp4 "Opening shot"` becomes `clip.mp4`.
fn strip_title(link: &str) -> &str {
    link.strip_suffix('"')
        .and_then(|link| link.rfind(" \"").map(|title| link[..title].trim_end()))
        .unwrap_or(link)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beats_split_at_media_after_narration() {
        let script = parse(
            "\u{feff}---\nfps: 24\n---\n![](a.mp4){dur=2 in=fade}\n> One two.\n> Three.\n![](b (1).png \"Map\")\n> Four.\n",
        )
        .unwrap();
        assert_eq!(script.settings["fps"], "24");
        assert_eq!(script.beats.len(), 2);
        assert_eq!(script.beats[0].text, ["One two.", "Three."]);
        assert_eq!(script.beats[0].media[0].attributes["in"], "fade");
        assert_eq!(script.beats[1].media[0].path, "b (1).png");
        assert_eq!(script.beats[1].line, 7);
    }
}
