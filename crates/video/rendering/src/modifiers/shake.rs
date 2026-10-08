use super::{RasterModifierRuntime, VectorModifierRuntime};
use crate::visual_source::VisualModifierContext;
use shrimply_evaluation::{resolve_scalar, resolve_vec2};
use shrimply_video_modifiers::shake::ShakeModifier;

fn shake(
    effect: &ShakeModifier,
    context: &mut VisualModifierContext<'_>,
) -> shrimply_math_geometry::ComposedTransform2D {
    let offset = resolve_vec2(&effect.offset, context.evaluation, context.expressions);
    let mut r = |value| resolve_scalar(value, context.evaluation, context.expressions);
    let shake = crate::math::Shake {
        amplitude: r(&effect.amplitude),
        offset,
        rotation_degrees: r(&effect.rotation_degrees),
        scale_percent: r(&effect.scale),
        frequency: r(&effect.frequency),
        seed: r(&effect.seed),
    };
    let canvas = context.project.canvas_size;
    let center = glam::Vec2::new(canvas.width as f32, canvas.height as f32) * 0.5;
    crate::math::shake_transform(shake, center, context.evaluation.local_time().seconds).composed()
}

impl VectorModifierRuntime for ShakeModifier {
    fn apply_vector(
        &self,
        mut input: crate::layer::VectorVisual,
        context: &mut VisualModifierContext<'_>,
    ) -> Result<crate::layer::VectorVisual, String> {
        input.push(crate::layer::VectorOperation::Transform(shake(
            self, context,
        )));
        Ok(input)
    }
}

impl RasterModifierRuntime for ShakeModifier {
    fn apply_raster(
        &self,
        mut input: crate::layer::RasterVisual,
        context: &mut VisualModifierContext<'_>,
    ) -> Result<crate::layer::RasterVisual, String> {
        let transform = shake(self, context);
        input.push_spatial(move |state| {
            state.transform = transform.compose(state.transform);
        });
        Ok(input)
    }
}
