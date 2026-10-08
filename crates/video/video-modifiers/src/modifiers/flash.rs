use super::{KeyframeSpan, ModifierModel, combine, ensure_timeline_value_ids, timeline_value_span};
use hashbrown::HashSet;
use serde::{Deserialize, Serialize};
use shrimply_core::timeline_value::*;
use uuid::Uuid;

/// Exposure bump followed by a mix toward a flash color, both meant to be keyframed on hits.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FlashModifier {
    /// Exposure change in stops.
    pub exposure: TimelineValue<f32>,
    /// Mix toward `color`, from zero to one.
    pub amount: TimelineValue<f32>,
    pub color: TimelineValue<shrimply_core::Color<u8>>,
}

impl Default for FlashModifier {
    fn default() -> Self {
        Self {
            exposure: TimelineValue::<f32>::new_const(1.0),
            amount: TimelineValue::<f32>::new_const(0.0),
            color: TimelineValue::new_const(shrimply_core::Color::<u8>::WHITE),
        }
    }
}

impl ModifierModel for FlashModifier {
    fn display_name(&self) -> &'static str {
        "Flash"
    }

    fn keywords(&self) -> &'static [&'static str] {
        &[
            "exposure",
            "bump",
            "strobe",
            "white flash",
            "impact",
            "beat",
        ]
    }

    fn ensure_ids(&mut self, seen: &mut HashSet<Uuid>) {
        ensure_timeline_value_ids(&mut self.exposure, seen);
        ensure_timeline_value_ids(&mut self.amount, seen);
        ensure_timeline_value_ids(&mut self.color, seen);
    }

    fn keyframe_span(&self) -> KeyframeSpan {
        combine([
            timeline_value_span(&self.exposure),
            timeline_value_span(&self.amount),
            timeline_value_span(&self.color),
        ])
    }

    fn number(&self, id: Uuid) -> Option<&TimelineValue<f32>> {
        [&self.exposure, &self.amount]
            .into_iter()
            .find(|value| value.id == id)
    }

    fn number_mut(&mut self, id: Uuid) -> Option<&mut TimelineValue<f32>> {
        [&mut self.exposure, &mut self.amount]
            .into_iter()
            .find(|value| value.id == id)
    }

    fn color_mut(&mut self, id: Uuid) -> Option<&mut TimelineValue<shrimply_core::Color<u8>>> {
        [&mut self.color].into_iter().find(|value| value.id == id)
    }
}
