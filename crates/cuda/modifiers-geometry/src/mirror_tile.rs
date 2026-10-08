use cuda_core::CudaContext;
use cuda_device::{DisjointSlice, cuda_module, kernel, thread};
use cuda_host::EmbeddedModuleError;
use shrimply_render_core::MirrorTileParams;
use std::sync::Arc;
pub(crate) fn load(c: &Arc<CudaContext>) -> Result<device::LoadedModule, EmbeddedModuleError> {
    device::load(c)
}
#[cuda_module]
pub(crate) mod device {
    use super::*;
    use crate::math;
    fn address(index: i32, length: u32, mirror: u32) -> u32 {
        if mirror != 0 {
            math::mirror_repeat_index(index, 0, length as i32) as u32
        } else {
            math::repeat_index(index, 0, length as i32) as u32
        }
    }
    #[kernel]
    pub fn mirror_tile(
        input: *const u32,
        w: u32,
        h: u32,
        mut out: DisjointSlice<u32>,
        params: MirrorTileParams,
    ) {
        let idx = thread::index_1d();
        let i = idx.get();
        let Some(o) = out.get_mut(idx) else {
            return;
        };
        let x = (i as u32 % w) as f32 + 0.5;
        let y = (i as u32 / w) as f32 + 0.5;
        let sx = params.center.x + (x - params.center.x) * params.tiles - 0.5;
        let sy = params.center.y + (y - params.center.y) * params.tiles - 0.5;
        let x0 = math::floor_f32(sx);
        let y0 = math::floor_f32(sy);
        let fx = sx - x0;
        let fy = sy - y0;
        let at = |px: f32, py: f32| {
            let px = address(px as i32, w, params.mirror_x);
            let py = address(py as i32, h, params.mirror_y);
            math::Color::from_rgba_u32(unsafe { *input.add((py * w + px) as usize) })
        };
        let top = at(x0, y0).lerp(at(x0 + 1.0, y0), fx);
        let bottom = at(x0, y0 + 1.0).lerp(at(x0 + 1.0, y0 + 1.0), fx);
        *o = top.lerp(bottom, fy).to_rgba_u32();
    }
}
