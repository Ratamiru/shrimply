use super::RasterModifierRuntime;
use crate::gpu::modifiers::{CanvasRgbaFrame, GpuModifier, ModifierContext};
use crate::layer::RasterVisual;
use crate::visual_source::VisualModifierContext;
use cuda_core::LaunchConfig;
use cuda_device::{DisjointSlice, kernel};
use shrimply_evaluation::{resolve_color, resolve_scalar};
use shrimply_math_color::Color;
use shrimply_video_modifiers::glow_bloom::{
    GlowBloomModifier, MAX_GLOW_LEVEL_RADIUS, MAX_GLOW_LEVELS, MAX_GLOW_RADIUS,
};

#[kernel]
fn glow_bloom_horizontal(_: *const u32, _: u32, _: DisjointSlice<Color>, _: f32, _: u32) {}

#[kernel]
fn glow_bloom_vertical(
    _: *const Color,
    _: u32,
    _: u32,
    _: DisjointSlice<Color>,
    _: u32,
    _: f32,
    _: u32,
) {
}

#[kernel]
fn glow_bloom_composite(_: *const u32, _: *const Color, _: DisjointSlice<u32>, _: u32, _: f32) {}

struct Resolved {
    threshold: f32,
    radius: u32,
    levels: u32,
    intensity: f32,
    tint: u32,
}
impl GpuModifier for Resolved {
    fn name(&self) -> &'static str {
        "Glow / bloom"
    }
    fn apply(
        &self,
        c: &mut ModifierContext<'_>,
        input: CanvasRgbaFrame,
    ) -> Result<CanvasRgbaFrame, String> {
        let width = input.width();
        let height = input.height();
        let count = width as usize * height as usize;
        let launch =
            LaunchConfig::for_num_elems(u32::try_from(count).map_err(|_| "canvas is too large")?);
        let mut pass = input.into_pass(c)?;
        let mut horizontal = c.take_typed_scratch::<Color>(count)?;
        let mut accumulated = c.take_typed_scratch::<Color>(count)?;
        let module = c.modifier_module(crate::gpu::modifiers::ModifierModule::Blur)?;
        let weight = 1.0 / self.levels as f32;
        for level in 0..self.levels {
            let radius = (self.radius << level).min(MAX_GLOW_LEVEL_RADIUS);
            unsafe {
                cuda_host::cuda_launch! {
                    kernel: glow_bloom_horizontal,
                    stream: c.stream(), module: &module, config: launch,
                    args: [
                        pass.input_ptr(), width, slice_mut(&mut horizontal),
                        self.threshold, radius
                    ]
                }
            }
            .map_err(|error| format!("launch horizontal CUDA kernel: {error:?}"))?;
            let horizontal_ptr = horizontal.cu_deviceptr() as usize as *const Color;
            unsafe {
                cuda_host::cuda_launch! {
                    kernel: glow_bloom_vertical,
                    stream: c.stream(), module: &module, config: launch,
                    args: [
                        horizontal_ptr, width, height, slice_mut(&mut accumulated),
                        radius, weight, u32::from(level == 0)
                    ]
                }
            }
            .map_err(|error| format!("launch vertical CUDA kernel: {error:?}"))?;
        }
        let accumulated_ptr = accumulated.cu_deviceptr() as usize as *const Color;
        unsafe {
            cuda_host::cuda_launch! {
                kernel: glow_bloom_composite,
                stream: c.stream(), module: &module, config: launch,
                args: [
                    pass.input_ptr(), accumulated_ptr,
                    slice_mut(pass.output_buffer()), self.tint, self.intensity
                ]
            }
        }
        .map_err(|error| format!("launch composite CUDA kernel: {error:?}"))?;
        c.recycle_typed_scratch(horizontal);
        c.recycle_typed_scratch(accumulated);
        Ok(pass.finish(c))
    }
}
impl RasterModifierRuntime for GlowBloomModifier {
    fn apply_raster(
        &self,
        mut input: RasterVisual,
        c: &mut VisualModifierContext<'_>,
    ) -> Result<RasterVisual, String> {
        let mut r = |value| resolve_scalar(value, c.evaluation, c.expressions);
        let threshold = r(&self.threshold).clamp(0., 1.);
        let radius = r(&self.radius).clamp(0., MAX_GLOW_RADIUS) as u32;
        let levels = r(&self.levels).round().clamp(1., MAX_GLOW_LEVELS) as u32;
        let intensity = r(&self.intensity).clamp(0., 5.);
        input.push_pixel(Box::new(Resolved {
            threshold,
            radius,
            levels,
            intensity,
            tint: resolve_color(&self.color, c.evaluation, c.expressions).to_rgba_u32(),
        }));
        Ok(input)
    }
}
