use super::RasterModifierRuntime;
use crate::gpu::modifiers::{CanvasRgbaFrame, GpuModifier, ModifierContext};
use crate::layer::RasterVisual;
use crate::visual_source::VisualModifierContext;
use cuda_core::LaunchConfig;
use cuda_device::{DisjointSlice, kernel};
use shrimply_evaluation::{resolve_color, resolve_scalar};
use shrimply_video_modifiers::flash::FlashModifier;

const MAX_EXPOSURE_STOPS: f32 = 10.0;

#[kernel]
fn flash(_: *const u32, _: DisjointSlice<u32>, _: f32, _: u32, _: f32) {}
struct Resolved {
    gain: f32,
    color: u32,
    amount: f32,
}
impl GpuModifier for Resolved {
    fn name(&self) -> &'static str {
        "Flash"
    }
    fn apply(
        &self,
        c: &mut ModifierContext<'_>,
        input: CanvasRgbaFrame,
    ) -> Result<CanvasRgbaFrame, String> {
        let n = input.width() as usize * input.height() as usize;
        let mut p = input.into_pass(c)?;
        let m = c.modifier_module(crate::gpu::modifiers::ModifierModule::General)?;
        unsafe {
            cuda_host::cuda_launch! {
                kernel: flash,
                stream: c.stream(),
                module: &m,
                config: LaunchConfig::for_num_elems(u32::try_from(n).map_err(|_| "canvas is too large")?),
                args: [
                    p.input_ptr(),
                    slice_mut(p.output_buffer()),
                    self.gain,
                    self.color,
                    self.amount
                ]
            }
        }
        .map_err(|error| format!("launch CUDA kernel: {error:?}"))?;
        Ok(p.finish(c))
    }
}
impl RasterModifierRuntime for FlashModifier {
    fn apply_raster(
        &self,
        mut input: RasterVisual,
        c: &mut VisualModifierContext<'_>,
    ) -> Result<RasterVisual, String> {
        let exposure = resolve_scalar(&self.exposure, c.evaluation, c.expressions)
            .clamp(-MAX_EXPOSURE_STOPS, MAX_EXPOSURE_STOPS);
        let amount = resolve_scalar(&self.amount, c.evaluation, c.expressions).clamp(0., 1.);
        if exposure == 0.0 && amount == 0.0 {
            return Ok(input);
        }
        input.push_pixel(Box::new(Resolved {
            gain: exposure.exp2(),
            color: resolve_color(&self.color, c.evaluation, c.expressions).to_rgba_u32(),
            amount,
        }));
        Ok(input)
    }
}
