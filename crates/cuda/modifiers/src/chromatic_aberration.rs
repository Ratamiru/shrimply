use cuda_core::CudaContext;
use cuda_device::{DisjointSlice, cuda_module, kernel, thread};
use cuda_host::EmbeddedModuleError;
use shrimply_render_core::{ChromaticAberrationMode, ChromaticAberrationParams};
use std::sync::Arc;
pub(crate) fn load(c: &Arc<CudaContext>) -> Result<device::LoadedModule, EmbeddedModuleError> {
    device::load(c)
}
#[cuda_module]
pub(crate) mod device {
    use super::*;
    use crate::math;
    fn clamp(v: i32, max: u32) -> u32 {
        v.max(0).min(max as i32 - 1) as u32
    }
    #[kernel]
    pub fn chromatic_aberration(
        input: *const u32,
        width: u32,
        height: u32,
        mut out: DisjointSlice<u32>,
        params: ChromaticAberrationParams,
    ) {
        let idx = thread::index_1d();
        let i = idx.get();
        let Some(d) = out.get_mut(idx) else { return };
        let x = i as u32 % width;
        let y = i as u32 / width;
        let [r, b] = if matches!(params.mode, ChromaticAberrationMode::Radial) {
            let center_x = params.center.x * width as f32;
            let center_y = params.center.y * height as f32;
            let dx = (x as f32 + 0.5 - center_x) * params.radial_strength;
            let dy = (y as f32 + 0.5 - center_y) * params.radial_strength;
            [
                bilinear(input, width, height, x as f32 + dx, y as f32 + dy).r,
                bilinear(input, width, height, x as f32 - dx, y as f32 - dy).b,
            ]
        } else {
            let at = |ox: f32, oy: f32| unsafe {
                *input.add(
                    (clamp(y as i32 + oy.round() as i32, height) * width
                        + clamp(x as i32 + ox.round() as i32, width)) as usize,
                )
            };
            [
                math::Color::from_rgba_u32(at(params.offsets[0], params.offsets[1])).r,
                math::Color::from_rgba_u32(at(params.offsets[2], params.offsets[3])).b,
            ]
        };
        let [_, g, _, a] = math::Color::from_rgba_u32(unsafe { *input.add(i) }).to_array();
        *d = math::Color::new(r, g, b, a).to_rgba_u32();
    }

    fn bilinear(input: *const u32, width: u32, height: u32, x: f32, y: f32) -> math::Color {
        let x0 = x.floor();
        let y0 = y.floor();
        let fx = x - x0;
        let fy = y - y0;
        let at = |sx: f32, sy: f32| {
            math::Color::from_rgba_u32(unsafe {
                *input.add((clamp(sy as i32, height) * width + clamp(sx as i32, width)) as usize)
            })
        };
        let top = at(x0, y0).lerp(at(x0 + 1.0, y0), fx);
        let bottom = at(x0, y0 + 1.0).lerp(at(x0 + 1.0, y0 + 1.0), fx);
        top.lerp(bottom, fy)
    }
}
