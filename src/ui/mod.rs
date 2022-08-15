pub mod control_panel_renderer;
pub mod map_loader;
pub mod map_mode;
pub mod right_panel_renderer;
pub mod root_path;
pub mod selection;
pub mod top_menu_renderer;

use crate::ui::control_panel_renderer::ControlPanelRenderer;
use crate::ui::map_mode::MapMode;
use crate::ui::right_panel_renderer::RightPanelRenderer;
use crate::ui::top_menu_renderer::TopMenuRenderer;
use actix::Addr;

pub struct UiRenderer {
    pub top_menu_renderer: Addr<TopMenuRenderer>,
    pub control_panel_renderer: Addr<ControlPanelRenderer>,
    pub right_panel_renderer: Addr<RightPanelRenderer>,
    pub map_mode: Addr<MapMode>,
}

impl UiRenderer {
    #[inline]
    pub const fn new(
        top_menu_renderer: Addr<TopMenuRenderer>,
        control_panel_renderer: Addr<ControlPanelRenderer>,
        right_panel_renderer: Addr<RightPanelRenderer>,
        map_mode: Addr<MapMode>,
    ) -> Self {
        Self {
            top_menu_renderer,
            control_panel_renderer,
            right_panel_renderer,
            map_mode,
        }
    }
}
