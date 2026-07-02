//! `RenderContext` — passed to `Behavior::render_self` during the execute pass.

use crate::renderable::theme::UiTheme;
use crate::renderer::HitGrid;
use crate::{GraphemePool, LinkPool, OptimizedBuffer};

#[derive(Debug)]
pub struct RenderContext<'a> {
    pub buffer: &'a mut OptimizedBuffer,
    pub grapheme_pool: Option<&'a mut GraphemePool>,
    pub link_pool: Option<&'a mut LinkPool>,
    pub hit_grid: Option<&'a mut HitGrid>,
    pub theme: Option<&'a UiTheme>,
}
