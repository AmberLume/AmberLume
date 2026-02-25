use anyhow::Result;
use anyhow::bail;
use glam::Mat4;

#[repr(u32)]
pub enum RenderViewArrayIndex {
    Main = 0,
    GlobalShadowCascades = 1,
}

pub struct RenderViewArray {
    pub index: RenderViewArrayIndex,
    pub items: Vec<RenderView>,
}

impl RenderViewArray {
    pub fn for_each<F>(&self, layout: &RenderViewsLayout, mut f: F) -> Result<()>
    where
        F: FnMut(u32, u32, &RenderView) -> Result<()>,
    {
        for (local_index, render_view) in self.items.iter().enumerate() {
            let global_index = layout.get_global_index_of(&self.index, local_index as u32)?;
            
            f(global_index, local_index as u32, render_view)?;
        }

        Ok(())
    }

    pub fn size(&self) -> u32 {
        self.items.len() as u32
    }
}

pub struct RenderView {
    pub projection_view: Mat4,
}

pub struct RenderViewsLayout {
    pub main: RenderViewArray,
    pub global_shadow_cascades: RenderViewArray,
}

impl RenderViewsLayout {
    pub fn count(&self) -> u32 {
        let mut count: u32 = 0;

        count += self.main.size();
        count += self.global_shadow_cascades.size();

        count
    }

    pub fn get_global_index_of(
        &self,
        render_view_array: &RenderViewArrayIndex,
        index: u32,
    ) -> Result<u32> {
        let mut global_index: u32 = 0;

        if let RenderViewArrayIndex::Main = render_view_array {
            return Ok(global_index + index)
        } else {
            global_index += self.main.size();
        }

        if let RenderViewArrayIndex::GlobalShadowCascades = render_view_array {
            return Ok(global_index + index)
        } else {
            global_index += self.global_shadow_cascades.size();
        }

        bail!("Requested index is out of bounds. Last RenderView index is {}", global_index);
    }
}
