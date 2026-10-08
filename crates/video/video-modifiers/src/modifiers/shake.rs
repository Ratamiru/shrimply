use super::{KeyframeSpan, ModifierModel, combine, ensure_timeline_value_ids, timeline_value_span};
use hashbrown::HashSet;
use serde::{Deserialize, Serialize};
use shrimply_core::timeline_value::*;
use uuid::Uuid;

/// Moves, rotates and scales the visual about the canvas center along smooth noise.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShakeModifier {
    /// Multiplies every extent; keyframe it to hit the shake on a beat.
    pub amplitude: TimelineValue<f32>,
    pub offset: TimelineValue<glam::Vec2>,
    pub rotation_degrees: TimelineValue<f32>,
    /// Scale change in percent.
    pub scale: TimelineValue<f32>,
    /// Noise targets per second.
    pub frequency: TimelineValue<f32>,
    pub seed: TimelineValue<f32>,
}

impl Default for ShakeModifier {
    fn default() -> Self {
        Self {
            amplitude: TimelineValue::<f32>::new_const(1.0),
            offset: TimelineValue::<glam::Vec2>::new_const(glam::Vec2::splat(20.0)),
            rotation_degrees: TimelineValue::<f32>::new_const(2.0),
            scale: TimelineValue::<f32>::new_const(2.0),
            frequency: TimelineValue::<f32>::new_const(12.0),
            seed: TimelineValue::<f32>::new_const(0.0),
        }
    }
}

impl ModifierModel for ShakeModifier {
    fn display_name(&self) -> &'static str {
        "Shake"
    }

    fn keywords(&self) -> &'static [&'static str] {
        &["camera shake", "wiggle", "jitter", "impact", "beat"]
    }

    fn ensure_ids(&mut self, seen: &mut HashSet<Uuid>) {
        ensure_timeline_value_ids(&mut self.amplitude, seen);
        ensure_timeline_value_ids(&mut self.offset, seen);
        ensure_timeline_value_ids(&mut self.rotation_degrees, seen);
        ensure_timeline_value_ids(&mut self.scale, seen);
        ensure_timeline_value_ids(&mut self.frequency, seen);
        ensure_timeline_value_ids(&mut self.seed, seen);
    }

    fn keyframe_span(&self) -> KeyframeSpan {
        combine([
            timeline_value_span(&self.amplitude),
            timeline_value_span(&self.offset),
            timeline_value_span(&self.rotation_degrees),
            timeline_value_span(&self.scale),
            timeline_value_span(&self.frequency),
            timeline_value_span(&self.seed),
        ])
    }

    fn number(&self, id: Uuid) -> Option<&TimelineValue<f32>> {
        [
            &self.amplitude,
            &self.rotation_degrees,
            &self.scale,
            &self.frequency,
            &self.seed,
        ]
        .into_iter()
        .find(|value| value.id == id)
    }

    fn number_mut(&mut self, id: Uuid) -> Option<&mut TimelineValue<f32>> {
        [
            &mut self.amplitude,
            &mut self.rotation_degrees,
            &mut self.scale,
            &mut self.frequency,
            &mut self.seed,
        ]
        .into_iter()
        .find(|value| value.id == id)
    }

    fn number2(&self, id: Uuid) -> Option<&TimelineValue<glam::Vec2>> {
        [&self.offset].into_iter().find(|value| value.id == id)
    }

    fn number2_mut(&mut self, id: Uuid) -> Option<&mut TimelineValue<glam::Vec2>> {
        [&mut self.offset].into_iter().find(|value| value.id == id)
    }
}
