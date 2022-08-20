pub mod central_panel_renderer;
pub mod control_panel_renderer;
pub mod map_loader;
pub mod map_mode;
pub mod map_textures;
pub mod right_panel_renderer;
pub mod root_path;
pub mod selection;
pub mod top_menu_renderer;
pub mod viewport;

use crate::ui::central_panel_renderer::CentralPanelRenderer;
use crate::ui::control_panel_renderer::ControlPanelRenderer;
use crate::ui::map_mode::MapMode;
use crate::ui::right_panel_renderer::RightPanelRenderer;
use crate::ui::top_menu_renderer::TopMenuRenderer;
use crate::ui::viewport::Viewport;
use actix::Addr;

pub struct UiRenderer {
    pub top_menu_renderer: TopMenuRenderer,
    pub control_panel_renderer: ControlPanelRenderer,
    pub right_panel_renderer: RightPanelRenderer,
    pub central_panel_renderer: CentralPanelRenderer,
    pub map_mode: Addr<MapMode>,
    pub viewport: Addr<Viewport>,
}

impl UiRenderer {
    #[inline]
    pub const fn new(
        top_menu_renderer: TopMenuRenderer,
        control_panel_renderer: ControlPanelRenderer,
        right_panel_renderer: RightPanelRenderer,
        central_panel_renderer: CentralPanelRenderer,
        map_mode: Addr<MapMode>,
        viewport: Addr<Viewport>,
    ) -> Self {
        Self {
            top_menu_renderer,
            control_panel_renderer,
            right_panel_renderer,
            central_panel_renderer,
            map_mode,
            viewport,
        }
    }
}
