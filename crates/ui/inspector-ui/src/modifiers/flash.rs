use super::{InspectorContext, ScalarOptions, color_row, scalar_row};
use gtk::prelude::*;
use shrimply_video_modifiers::flash::FlashModifier;
use uuid::Uuid;

pub fn add_rows(value: &FlashModifier, out: &gtk::Box, id: Uuid, context: &InspectorContext) {
    out.append(&scalar_row(
        "Exposure",
        &value.exposure,
        id,
        ScalarOptions {
            minimum: Some(-10.0),
            maximum: Some(10.0),
            unit: Some("EV"),
            rotating: false,
        },
        context,
    ));
    out.append(&scalar_row(
        "Amount",
        &value.amount,
        id,
        ScalarOptions {
            minimum: Some(0.0),
            maximum: Some(1.0),
            unit: None,
            rotating: false,
        },
        context,
    ));
    out.append(&color_row("Color", &value.color, id, context));
}
