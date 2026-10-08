use super::{KeyframeSpan, ModifierModel, combine, ensure_timeline_value_ids, timeline_value_span};
use hashbrown::HashSet;
use serde::{Deserialize, Serialize};
use shrimply_core::timeline_value::*;
use uuid::Uuid;

pub const MAX_GLOW_RADIUS: f32 = 32.0;
pub const MAX_GLOW_LEVELS: f32 = 6.0;
/// Upper bound for the radius of the widest level, keeping box-blur passes affordable.
pub const MAX_GLOW_LEVEL_RADIUS: u32 = 256;

fn default_color() -> TimelineValue<shrimply_core::Color<u8>> {
    TimelineValue::new_const(shrimply_core::Color::<u8>::WHITE)
}

fn default_levels() -> TimelineValue<f32> {
    TimelineValue::new_const(1.0)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlowBloomModifier {
    pub threshold: TimelineValue<f32>,
    pub radius: TimelineValue<f32>,
    pub intensity: TimelineValue<f32>,
    /// Number of blur octaves; each level doubles the radius of the previous one.
    #[serde(default = "default_levels")]
    pub levels: TimelineValue<f32>,
    #[serde(default = "default_color")]
    pub color: TimelineValue<shrimply_core::Color<u8>>,
}

impl Default for GlowBloomModifier {
    fn default() -> Self {
        Self {
            threshold: TimelineValue::<f32>::new_const(0.7),
            radius: TimelineValue::<f32>::new_const(10.0),
            intensity: TimelineValue::<f32>::new_const(1.0),
            levels: TimelineValue::<f32>::new_const(3.0),
            color: default_color(),
        }
    }
}

impl ModifierModel for GlowBloomModifier {
    fn display_name(&self) -> &'static str {
        "Glow"
    }

    fn keywords(&self) -> &'static [&'static str] {
        &["bloom", "glare", "halo", "light spill"]
    }

    fn ensure_ids(&mut self, seen: &mut HashSet<Uuid>) {
        ensure_timeline_value_ids(&mut self.threshold, seen);
        ensure_timeline_value_ids(&mut self.radius, seen);
        ensure_timeline_value_ids(&mut self.intensity, seen);
        ensure_timeline_value_ids(&mut self.levels, seen);
        ensure_timeline_value_ids(&mut self.color, seen);
    }

    fn keyframe_span(&self) -> KeyframeSpan {
        combine([
            timeline_value_span(&self.threshold),
            timeline_value_span(&self.radius),
            timeline_value_span(&self.intensity),
            timeline_value_span(&self.levels),
            timeline_value_span(&self.color),
        ])
    }

    fn number(&self, id: Uuid) -> Option<&TimelineValue<f32>> {
        [&self.threshold, &self.radius, &self.intensity, &self.levels]
            .into_iter()
            .find(|value| value.id == id)
    }

    fn number_mut(&mut self, id: Uuid) -> Option<&mut TimelineValue<f32>> {
        [
            &mut self.threshold,
            &mut self.radius,
            &mut self.intensity,
            &mut self.levels,
        ]
        .into_iter()
        .find(|value| value.id == id)
    }

    fn color_mut(&mut self, id: Uuid) -> Option<&mut TimelineValue<shrimply_core::Color<u8>>> {
        [&mut self.color].into_iter().find(|value| value.id == id)
    }
}
