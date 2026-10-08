use super::{InspectorContext, ScalarOptions, scalar_row, vec_row};
use gtk::prelude::*;
use shrimply_video_modifiers::{
    ModifierEffect, RasterModifierEffect,
    chromatic_aberration::{ChromaticAberrationMode, ChromaticAberrationModifier},
};
use uuid::Uuid;

pub fn add_rows(
    value: &ChromaticAberrationModifier,
    out: &gtk::Box,
    id: Uuid,
    context: &InspectorContext,
) {
    let position = crate::player_state::snapshot(&context.player_state).position;
    let local_time = context
        .selected_item
        .clone()
        .and_then(|key| crate::video::visual_local_time(&context.project.borrow(), key, position))
        .unwrap_or(shrimply_project::project::Time::ZERO);
    out.append(&super::step_row(
        "Mode",
        &value.mode,
        id,
        context,
        "edit-chromatic-aberration-mode",
        |modifier| match modifier {
            ModifierEffect::Raster(effect) => match &**effect {
                RasterModifierEffect::ChromaticAberration(effect) => Some(&effect.mode),
                _ => None,
            },
            _ => None,
        },
        |modifier| match modifier {
            ModifierEffect::Raster(effect) => match &mut **effect {
                RasterModifierEffect::ChromaticAberration(effect) => Some(&mut effect.mode),
                _ => None,
            },
            _ => None,
        },
    ));
    if value.mode.value_at(local_time) == ChromaticAberrationMode::Radial {
        out.append(&vec_row(
            "Center",
            &value.center,
            id,
            true,
            Some((0.0, 1.0)),
            context,
        ));
        out.append(&scalar_row(
            "Strength",
            &value.radial_strength,
            id,
            ScalarOptions {
                minimum: Some(-50.0),
                maximum: Some(50.0),
                unit: Some("%"),
                rotating: false,
            },
            context,
        ));
        return;
    }
    out.append(&scalar_row(
        "Red X",
        &value.red_offset_x,
        id,
        ScalarOptions {
            minimum: None,
            maximum: None,
            unit: Some("px"),
            rotating: false,
        },
        context,
    ));
    out.append(&scalar_row(
        "Red Y",
        &value.red_offset_y,
        id,
        ScalarOptions {
            minimum: None,
            maximum: None,
            unit: Some("px"),
            rotating: false,
        },
        context,
    ));
    out.append(&scalar_row(
        "Blue X",
        &value.blue_offset_x,
        id,
        ScalarOptions {
            minimum: None,
            maximum: None,
            unit: Some("px"),
            rotating: false,
        },
        context,
    ));
    out.append(&scalar_row(
        "Blue Y",
        &value.blue_offset_y,
        id,
        ScalarOptions {
            minimum: None,
            maximum: None,
            unit: Some("px"),
            rotating: false,
        },
        context,
    ));
}
