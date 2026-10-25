//! Builds a Shrimply project from a Markdown script.
//!
//! The script is the source of truth for structure and timing: every quote
//! paragraph becomes a caption and sets how long its beat lasts, and the media
//! linked above it fills that time. Fine motion work happens in the editor.

mod syntax;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use ffmpeg_next as ffmpeg;
use glam::Vec2;
use serde_json::{Value, json};
use shrimply_core::timeline_value::{TimelineBase, TimelineValue, TimelineValueType};
use shrimply_math_core::{
    Fraction, fraction_denominator, fraction_new, fraction_numerator, time_from_frame,
};
use shrimply_project::caption::CaptionItem;
use shrimply_project::project::{
    AudioItem, CanvasSize, PROJECT_FORMAT_VERSION, Time, Transform, TransitionSide, VideoItem,
    VisualSource, VisualTransition, VisualTransitionKind, VisualTransitions, native_playback_fps,
};

use syntax::{Beat, Media};

const DEFAULT_CANVAS: CanvasSize = CanvasSize {
    width: 1920,
    height: 1080,
};
const DEFAULT_FPS: u32 = 30;
const DEFAULT_WORDS_PER_MINUTE: i64 = 150;
const SECONDS_PER_MINUTE: i64 = 60;
const MILLISECONDS_PER_SECOND: i64 = 1000;
/// How long an image or unreadable clip stays up when no text times it.
const DEFAULT_STILL_MS: i64 = 3000;
/// Shortest beat for a line of text, so one-word lines stay readable.
const MIN_TEXT_MS: i64 = 1000;
/// Shortest share of a beat for an auto-timed shot.
const MIN_AUTO_SHOT_MS: i64 = 500;
const DEFAULT_TRANSITION_MS: i64 = 500;
const AUDIO_EXTENSIONS: [&str; 7] = ["aac", "flac", "m4a", "mp3", "ogg", "opus", "wav"];
const SETTING_KEYS: [&str; 4] = ["captions", "fps", "size", "wpm"];

pub struct ImportResult {
    pub project: Value,
    pub warnings: Vec<String>,
}

struct Settings {
    canvas: CanvasSize,
    fps: Fraction,
    words_per_minute: i64,
    captions: bool,
}

/// One visual link, resolved and given its slot on the timeline.
struct Shot<'a> {
    media: &'a Media,
    path: PathBuf,
    content: VisualSource,
    /// Offset into the source.
    at: Time,
    /// Playable source length after `at`, for clips only.
    natural: Option<Time>,
    start: Time,
    end: Time,
}

/// Everything placed so far while walking the beats.
///
/// Shots become items only after every beat is placed, because narration can still stretch the
/// last shot and its zoom and transitions must fit the final length.
struct Timeline<'a> {
    cursor: Time,
    frame_step: Time,
    shots: Vec<Shot<'a>>,
    audio_tracks: Vec<Vec<AudioItem>>,
    captions: Vec<CaptionItem>,
    warnings: Vec<String>,
}

pub fn from_file(path: &Path) -> Result<ImportResult, String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    let script = syntax::parse(&source)?;
    let settings = settings(&script.settings)?;
    let base = path.parent().unwrap_or_else(|| Path::new("."));
    let frame_step =
        time_from_frame(1, settings.fps).ok_or_else(|| "invalid script fps".to_string())?;
    let mut timeline = Timeline {
        cursor: Time::ZERO,
        frame_step,
        shots: Vec::new(),
        audio_tracks: Vec::new(),
        captions: Vec::new(),
        warnings: Vec::new(),
    };
    for key in script.settings.keys() {
        if !SETTING_KEYS.contains(&key.as_str()) {
            timeline
                .warnings
                .push(format!("unknown front matter key `{key}` was ignored"));
        }
    }
    ffmpeg::init().map_err(|error| format!("could not initialise ffmpeg: {error}"))?;
    for beat in &script.beats {
        place_beat(beat, base, &settings, &mut timeline)?;
    }
    let mut visual = Vec::with_capacity(timeline.shots.len());
    for shot in timeline.shots {
        visual.push(visual_item(shot, settings.canvas, &mut timeline.warnings)?);
    }

    let name = path
        .file_stem()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Script".to_string());
    let audio_tracks: Vec<Value> = timeline
        .audio_tracks
        .into_iter()
        .map(|items| json!({ "enabled": true, "gain_db": 0.0, "items": items }))
        .collect();
    let caption_tracks = if timeline.captions.is_empty() {
        Vec::new()
    } else {
        vec![json!({ "enabled": true, "language": null, "items": timeline.captions })]
    };
    Ok(ImportResult {
        project: json!({
            "format_version": PROJECT_FORMAT_VERSION,
            "name": name,
            "fps": {
                "numerator": fraction_numerator(settings.fps),
                "denominator": fraction_denominator(settings.fps),
            },
            "canvas_size": settings.canvas,
            "caption_tracks": caption_tracks,
            "visual_tracks": [{ "enabled": true, "items": visual }],
            "audio_tracks": audio_tracks,
        }),
        warnings: timeline.warnings,
    })
}

fn place_beat<'a>(
    beat: &'a Beat,
    base: &Path,
    settings: &Settings,
    timeline: &mut Timeline<'a>,
) -> Result<(), String> {
    let beat_start = timeline.cursor;
    let words: usize = beat
        .text
        .iter()
        .map(|line| line.split_whitespace().count())
        .sum();
    let text_time = if words == 0 {
        Time::ZERO
    } else {
        let words = i64::try_from(words).map_err(|_| "beat has too many words".to_string())?;
        Time {
            seconds: fraction_new(words * SECONDS_PER_MINUTE, settings.words_per_minute),
        }
        .max(milliseconds(MIN_TEXT_MS))
    };

    let (audio, visual): (Vec<&Media>, Vec<&Media>) =
        beat.media.iter().partition(|media| is_audio(&media.path));
    let mut explicit = Vec::with_capacity(visual.len());
    for media in &visual {
        explicit.push(duration_attribute(media)?);
    }
    let explicit_total = explicit
        .iter()
        .flatten()
        .fold(Time::ZERO, |total, length| total.saturating_add(*length));
    let auto_count = explicit.iter().filter(|length| length.is_none()).count();
    let auto_share = (auto_count > 0 && words > 0).then(|| {
        let count = Fraction::from(auto_count as u64);
        Time {
            seconds: text_time.saturating_sub(explicit_total).seconds / count,
        }
        .max(milliseconds(MIN_AUTO_SHOT_MS))
    });

    for (media, length) in visual.into_iter().zip(explicit) {
        let path = resolve(base, &media.path);
        let content = shrimply_otio::visual_source(&path);
        let at = time_attribute(media, "at")?.unwrap_or(Time::ZERO);
        let natural = matches!(content, VisualSource::Media | VisualSource::Gif)
            .then(|| probe_duration(&path))
            .flatten()
            .map(|length| length.saturating_sub(at));
        let length = match (length, auto_share) {
            (Some(length), _) => length,
            (None, Some(share)) => share,
            (None, None) => natural.unwrap_or_else(|| milliseconds(DEFAULT_STILL_MS)),
        };
        let start = timeline.cursor;
        let end = snap_end(start, length, timeline.frame_step);
        timeline.shots.push(Shot {
            media,
            path,
            content,
            at,
            natural,
            start,
            end,
        });
        timeline.cursor = end;
    }

    // Narration longer than its shots holds the last shot until it is read.
    let narration_end = snap_end(beat_start, text_time, timeline.frame_step);
    if words > 0 && timeline.cursor < narration_end {
        match timeline.shots.last_mut() {
            Some(shot) => shot.end = narration_end,
            None => timeline.warnings.push(format!(
                "line {}: narration before any media plays over an empty frame",
                beat.line
            )),
        }
        timeline.cursor = narration_end;
    }

    for media in audio {
        place_audio(media, base, beat_start, timeline)?;
    }
    if settings.captions && words > 0 {
        timeline.captions.push(CaptionItem::new(
            beat_start,
            timeline.cursor,
            beat.text.join(" "),
        ));
    }
    Ok(())
}

fn visual_item(
    shot: Shot,
    canvas: CanvasSize,
    warnings: &mut Vec<String>,
) -> Result<VideoItem, String> {
    let Shot {
        media,
        path,
        content,
        at,
        natural,
        start,
        end,
    } = shot;
    warn_missing(media, &path, warnings);
    let size = shrimply_otio::visual_dimensions(&path, &content).unwrap_or_else(|| {
        warnings.push(format!(
            "line {}: could not read the size of {}; it fills the frame",
            media.line,
            path.display()
        ));
        glam::UVec2::new(canvas.width, canvas.height)
    });
    let transform = fitted_transform(media, canvas, size)?;
    let mut item = VideoItem::background_item(canvas, start, end);
    item.time_offset = at;
    item.source_duration = match natural {
        Some(natural) => natural.saturating_add(at),
        None => end.saturating_sub(start),
    };
    item.playback_fps = native_playback_fps();
    item.source_width = size.x;
    item.source_height = size.y;
    item.default_transform = Some(transform.clone());
    item.transform = transform;
    item.content = content;
    item.file = path.as_path().into();
    item.track_id = 0;
    if let Some(range) = media.attributes.get("zoom") {
        zoom(&mut item, range, media.line)?;
    }
    let length = end.saturating_sub(start);
    // Two transitions share the shot, so each gets at most half of it.
    let limit = if media.attributes.contains_key("in") && media.attributes.contains_key("out") {
        Time {
            seconds: length.seconds / Fraction::from(2u32),
        }
    } else {
        length
    };
    item.transitions = VisualTransitions {
        intro: transition(media, "in", TransitionSide::Intro, limit, canvas, warnings)?,
        outro: transition(media, "out", TransitionSide::Outro, limit, canvas, warnings)?,
        to_next: None,
    };
    Ok(item)
}

fn fitted_transform(
    media: &Media,
    canvas: CanvasSize,
    size: glam::UVec2,
) -> Result<Transform, String> {
    Ok(match media.attributes.get("fit").map(String::as_str) {
        None | Some("cover") => Transform::cover(canvas, size.x, size.y),
        Some("contain") => Transform::contain(canvas, size.x, size.y),
        Some("natural") => Transform::natural_size(canvas, size.x, size.y),
        Some("stretch") => Transform::stretch(canvas, size.x, size.y),
        Some(other) => {
            return Err(format!(
                "line {}: fit must be cover, contain, natural or stretch, not `{other}`",
                media.line
            ));
        }
    })
}

/// Ken Burns move: scales the fitted size from `a` to `b` over the shot.
fn zoom(item: &mut VideoItem, range: &str, line: usize) -> Result<(), String> {
    let parse = |value: &str| {
        value
            .parse::<f32>()
            .ok()
            .filter(|scale| scale.is_finite() && *scale > 0.0)
            .ok_or_else(|| format!("line {line}: zoom must look like 1..1.3"))
    };
    let (from, to) = range
        .split_once("..")
        .ok_or_else(|| format!("line {line}: zoom must look like 1..1.3"))?;
    let fitted = item.transform.scale.fallback();
    let length = item.end.saturating_sub(item.start);
    item.transform.scale = TimelineValue::new(TimelineBase::Keyframes(vec![
        Vec2::keyframe(Time::ZERO, fitted * parse(from)?),
        Vec2::keyframe(length, fitted * parse(to)?),
    ]));
    Ok(())
}

fn transition(
    media: &Media,
    key: &str,
    side: TransitionSide,
    limit: Time,
    canvas: CanvasSize,
    warnings: &mut Vec<String>,
) -> Result<Option<VisualTransition>, String> {
    let Some(name) = media.attributes.get(key) else {
        return Ok(None);
    };
    let kind: VisualTransitionKind = serde_json::from_value(Value::String(name.clone()))
        .map_err(|_| format!("line {}: unknown transition `{name}`", media.line))?;
    if kind.needs_vector_source() {
        return Err(format!(
            "line {}: `{name}` only works on text, shapes and drawings, not on media",
            media.line
        ));
    }
    let requested =
        time_attribute(media, "t")?.unwrap_or_else(|| milliseconds(DEFAULT_TRANSITION_MS));
    if requested > limit {
        warnings.push(format!(
            "line {}: the `{name}` transition was shortened to fit the shot",
            media.line
        ));
    }
    let mut transition = VisualTransition::new(side, requested.min(limit), canvas);
    transition.set_kind(side, kind);
    Ok(Some(transition))
}

fn place_audio(
    media: &Media,
    base: &Path,
    start: Time,
    timeline: &mut Timeline,
) -> Result<(), String> {
    let path = resolve(base, &media.path);
    warn_missing(media, &path, &mut timeline.warnings);
    let at = time_attribute(media, "at")?.unwrap_or(Time::ZERO);
    let natural = probe_duration(&path);
    let length = match (duration_attribute(media)?, natural) {
        (Some(length), _) => length,
        (None, Some(natural)) => natural.saturating_sub(at),
        (None, None) => {
            return Err(format!(
                "line {}: could not read the length of {}; give it dur=",
                media.line,
                path.display()
            ));
        }
    };
    let end = snap_end(start, length, timeline.frame_step);
    let item = AudioItem::builder(start, end)
        .time_offset(at)
        .source_duration(natural.unwrap_or_else(|| at.saturating_add(length)))
        .file(path)
        .build();
    // Overlapping sounds (voice over music) each get the first free track.
    match timeline
        .audio_tracks
        .iter_mut()
        .find(|track| track.last().is_none_or(|last| last.end <= start))
    {
        Some(track) => track.push(item),
        None => timeline.audio_tracks.push(vec![item]),
    }
    Ok(())
}

fn settings(values: &BTreeMap<String, String>) -> Result<Settings, String> {
    let canvas = match values.get("size") {
        None => DEFAULT_CANVAS,
        Some(size) => {
            let (width, height) = size
                .split_once('x')
                .and_then(|(width, height)| Some((width.parse().ok()?, height.parse().ok()?)))
                .filter(|(width, height): &(u32, u32)| *width > 0 && *height > 0)
                .ok_or_else(|| format!("size must look like 1920x1080, not `{size}`"))?;
            CanvasSize { width, height }
        }
    };
    let fps = match values.get("fps") {
        None => Fraction::from(DEFAULT_FPS),
        Some(fps) => fps
            .parse::<Fraction>()
            .ok()
            .filter(|fps| *fps > Fraction::from(0u32))
            .ok_or_else(|| format!("fps must be positive, not `{fps}`"))?,
    };
    let words_per_minute = match values.get("wpm") {
        None => DEFAULT_WORDS_PER_MINUTE,
        Some(wpm) => wpm
            .parse::<i64>()
            .ok()
            .filter(|wpm| *wpm > 0)
            .ok_or_else(|| format!("wpm must be a positive number, not `{wpm}`"))?,
    };
    let captions = match values.get("captions").map(String::as_str) {
        None | Some("true") => true,
        Some("false") => false,
        Some(other) => return Err(format!("captions must be true or false, not `{other}`")),
    };
    Ok(Settings {
        canvas,
        fps,
        words_per_minute,
        captions,
    })
}

fn duration_attribute(media: &Media) -> Result<Option<Time>, String> {
    if media
        .attributes
        .get("dur")
        .is_some_and(|value| value == "auto")
    {
        return Ok(None);
    }
    time_attribute(media, "dur")
}

/// Seconds as an integer, decimal or ratio: `4`, `2.5`, `1/3`.
fn time_attribute(media: &Media, key: &str) -> Result<Option<Time>, String> {
    let Some(value) = media.attributes.get(key) else {
        return Ok(None);
    };
    let seconds = value
        .trim_end_matches('s')
        .parse::<Fraction>()
        .ok()
        .filter(|seconds| *seconds >= Fraction::from(0u32))
        .ok_or_else(|| format!("line {}: {key} must be seconds, not `{value}`", media.line))?;
    Ok(Some(Time { seconds }))
}

fn probe_duration(path: &Path) -> Option<Time> {
    let input = ffmpeg::format::input(path).ok()?;
    let microseconds = input.duration();
    (microseconds > 0).then(|| Time {
        seconds: fraction_new(microseconds, i64::from(ffmpeg::ffi::AV_TIME_BASE)),
    })
}

fn snap_end(start: Time, length: Time, frame_step: Time) -> Time {
    start
        .saturating_add(length)
        .snapped(frame_step)
        .max(start.saturating_add(frame_step))
}

fn milliseconds(value: i64) -> Time {
    Time {
        seconds: fraction_new(value, MILLISECONDS_PER_SECOND),
    }
}

fn is_audio(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            AUDIO_EXTENSIONS
                .iter()
                .any(|candidate| extension.eq_ignore_ascii_case(candidate))
        })
}

fn resolve(base: &Path, path: &str) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        base.join(path)
    }
}

fn warn_missing(media: &Media, path: &Path, warnings: &mut Vec<String>) {
    if !path.exists() {
        warnings.push(format!(
            "line {}: {} does not exist",
            media.line,
            path.display()
        ));
    }
}
