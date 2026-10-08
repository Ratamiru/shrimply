use cuda_core::CudaContext;
use cuda_device::{DisjointSlice, cuda_module, kernel, thread};
use cuda_host::EmbeddedModuleError;
use std::sync::Arc;
pub(crate) fn load(c: &Arc<CudaContext>) -> Result<device::LoadedModule, EmbeddedModuleError> {
    device::load(c)
}
#[cuda_module]
pub(crate) mod device {
    use super::*;
    use crate::math;
    #[kernel]
    pub fn flash(
        input: *const u32,
        mut out: DisjointSlice<u32>,
        gain: f32,
        color: u32,
        amount: f32,
    ) {
        let idx = thread::index_1d();
        let i = idx.get();
        let Some(d) = out.get_mut(idx) else { return };
        let [r, g, b, a] = math::Color::from_rgba_u32(unsafe { *input.add(i) }).to_array();
        let flash = math::Color::from_rgba_u32(color);
        let mix = amount * flash.a;
        *d = math::Color::new(
            math::lerp(r * gain, flash.r, mix),
            math::lerp(g * gain, flash.g, mix),
            math::lerp(b * gain, flash.b, mix),
            a,
        )
        .to_rgba_u32();
    }
}
