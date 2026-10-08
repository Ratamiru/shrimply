use super::{InspectorContext, ScalarOptions, integer_scalar_row, scalar_row, vec_row};
use gtk::prelude::*;
use shrimply_video_modifiers::shake::ShakeModifier;
use uuid::Uuid;

pub fn add_rows(value: &ShakeModifier, out: &gtk::Box, id: Uuid, context: &InspectorContext) {
    out.append(&scalar_row(
        "Amplitude",
        &value.amplitude,
        id,
        ScalarOptions {
            minimum: Some(0.0),
            maximum: None,
            unit: None,
            rotating: false,
        },
        context,
    ));
    out.append(&vec_row("Offset", &value.offset, id, false, None, context));
    out.append(&scalar_row(
        "Rotation",
        &value.rotation_degrees,
        id,
        ScalarOptions {
            minimum: Some(0.0),
            maximum: None,
            unit: Some("deg"),
            rotating: false,
        },
        context,
    ));
    out.append(&scalar_row(
        "Scale",
        &value.scale,
        id,
        ScalarOptions {
            minimum: Some(0.0),
            maximum: Some(100.0),
            unit: Some("%"),
            rotating: false,
        },
        context,
    ));
    out.append(&scalar_row(
        "Frequency",
        &value.frequency,
        id,
        ScalarOptions {
            minimum: Some(0.0),
            maximum: Some(240.0),
            unit: Some("Hz"),
            rotating: false,
        },
        context,
    ));
    out.append(&integer_scalar_row(
        "Seed",
        &value.seed,
        id,
        ScalarOptions {
            minimum: Some(0.0),
            maximum: None,
            unit: None,
            rotating: false,
        },
        context,
    ));
}
