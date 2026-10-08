use super::RasterModifierRuntime;
use crate::gpu::modifiers::{CanvasRgbaFrame, GpuModifier, ModifierContext};
use crate::layer::RasterVisual;
use crate::visual_source::VisualModifierContext;
use cuda_core::LaunchConfig;
use cuda_device::{DisjointSlice, kernel};
use shrimply_evaluation::{resolve_scalar, resolve_vec2};
use shrimply_render_core::{ChromaticAberrationMode, ChromaticAberrationParams};
use shrimply_video_modifiers::chromatic_aberration::{self as model, ChromaticAberrationModifier};
#[kernel]
fn chromatic_aberration(
    _: *const u32,
    _: u32,
    _: u32,
    _: DisjointSlice<u32>,
    _: ChromaticAberrationParams,
) {
}
/// Radial strength is entered in percent of the distance from the center.
const PERCENT: f32 = 100.0;
const MAX_RADIAL_STRENGTH_PERCENT: f32 = 50.0;
const MAX_OFFSET: f32 = 4096.0;

struct Resolved {
    offsets: [f32; 4],
    mode: ChromaticAberrationMode,
    center: glam::Vec2,
    radial_strength: f32,
}
impl GpuModifier for Resolved {
    fn name(&self) -> &'static str {
        "Chromatic aberration"
    }
    fn apply(
        &self,
        c: &mut ModifierContext<'_>,
        input: CanvasRgbaFrame,
    ) -> Result<CanvasRgbaFrame, String> {
        let (w, h) = (input.width(), input.height());
        let n = w as usize * h as usize;
        let mut p = input.into_pass(c)?;
        let m = c.modifier_module(crate::gpu::modifiers::ModifierModule::General)?;
        let params = ChromaticAberrationParams {
            offsets: self.offsets,
            mode: self.mode,
            center: self.center,
            radial_strength: self.radial_strength,
        };
        unsafe {
            cuda_host::cuda_launch! {
                kernel: chromatic_aberration,
                stream: c.stream(),
                module: &m,
                config: LaunchConfig::for_num_elems(u32::try_from(n).map_err(|_| "canvas is too large")?),
                args: [
                    p.input_ptr(),
                    w,
                    h,
                    slice_mut(p.output_buffer()),
                    params
                ]
            }
        }
        .map_err(|error| format!("launch CUDA kernel: {error:?}"))?;
        Ok(p.finish(c))
    }
}
impl RasterModifierRuntime for ChromaticAberrationModifier {
    fn apply_raster(
        &self,
        mut input: RasterVisual,
        c: &mut VisualModifierContext<'_>,
    ) -> Result<RasterVisual, String> {
        let mut r = |x| resolve_scalar(x, c.evaluation, c.expressions);
        let offsets = [
            r(&self.red_offset_x),
            r(&self.red_offset_y),
            r(&self.blue_offset_x),
            r(&self.blue_offset_y),
        ]
        .map(|offset| offset.clamp(-MAX_OFFSET, MAX_OFFSET));
        let radial_strength = r(&self.radial_strength)
            .clamp(-MAX_RADIAL_STRENGTH_PERCENT, MAX_RADIAL_STRENGTH_PERCENT)
            / PERCENT;
        let mode = match self.mode.value_at(c.evaluation.local_time()) {
            model::ChromaticAberrationMode::Linear => ChromaticAberrationMode::Linear,
            model::ChromaticAberrationMode::Radial => ChromaticAberrationMode::Radial,
        };
        input.push_pixel(Box::new(Resolved {
            offsets,
            mode,
            center: resolve_vec2(&self.center, c.evaluation, c.expressions)
                .clamp(glam::Vec2::ZERO, glam::Vec2::ONE),
            radial_strength,
        }));
        Ok(input)
    }
}
