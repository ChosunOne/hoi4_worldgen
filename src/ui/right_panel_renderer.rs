use crate::ui::map_loader::GetMap;
use crate::ui::map_mode::GetMapMode;
use crate::ui::selection::{GetSelectedPoint, GetSelectedProvince, Selection, SetSelectedProvince};
use crate::{MapError, MapLoader, MapMode};
use actix::Addr;
use egui::{Context, Pos2, SidePanel, TopBottomPanel};
use indicatif::InMemoryTerm;
use log::{debug, trace};
use world_gen::components::prelude::Definition;
use world_gen::components::wrappers::Continent;
use world_gen::map::{
    GetContinentFromIndex, GetProvinceDefinitionFromId, GetProvinceIdFromPoint, Map,
};
use world_gen::MapDisplayMode;

pub struct RightPanelRenderer {
    map_mode: Addr<MapMode>,
    selection: Addr<Selection>,
    map_loader: Addr<MapLoader>,
    terminal: InMemoryTerm,
}

impl RightPanelRenderer {
    #[inline]
    pub const fn new(
        map_mode: Addr<MapMode>,
        selection: Addr<Selection>,
        map_loader: Addr<MapLoader>,
        terminal: InMemoryTerm,
    ) -> Self {
        Self {
            map_mode,
            selection,
            map_loader,
            terminal,
        }
    }

    pub async fn render_right_panel(&self, ctx: &Context) -> Result<(), MapError> {
        let map_mode: MapDisplayMode = self.map_mode.send(GetMapMode).await?;
        let map_addr: Option<Addr<Map>> = self.map_loader.send(GetMap).await?;
        let selected_point: Option<Pos2> = self.selection.send(GetSelectedPoint).await?;
        let selected_province: Option<Definition> =
            self.selection.send(GetSelectedProvince).await?;
        if let (Some(p), None, Some(m)) =
            (selected_point, selected_province.clone(), map_addr.clone())
        {
            // TODO: Perhaps reconsider where this logic should live
            if let Some(province_id) = m.send(GetProvinceIdFromPoint::new(p)).await? {
                if let Some(def) = m
                    .send(GetProvinceDefinitionFromId::new(province_id))
                    .await?
                {
                    self.selection.send(SetSelectedProvince::new(def)).await?;
                }
            }
        }
        let continent_index = selected_province
            .clone()
            .map(|d| d.continent)
            .filter(|c| c.0 > 0);
        let continent: Option<Continent> =
            if let (Some(c), Some(m)) = (continent_index, map_addr.clone()) {
                m.send(GetContinentFromIndex::new(c)).await?
            } else {
                None
            };
        SidePanel::right("right_panel")
            .resizable(false)
            .min_width(200.0)
            .show(ctx, |ui| {
                TopBottomPanel::top("info_panel")
                    .min_height(200.0)
                    .show_inside(ui, |ui| match map_mode {
                        MapDisplayMode::HeightMap => {}
                        MapDisplayMode::Terrain => {}
                        MapDisplayMode::Provinces => {
                            ui.label("Province Information");
                            if let (Some(_), Some(_), Some(definition)) =
                                (map_addr, selected_point, selected_province)
                            {
                                ui.label(format!("Id: {:?}", definition.id.0));
                                ui.label(format!(
                                    "Color: ({:?}, {:?}, {:?})",
                                    definition.r.0, definition.g.0, definition.b.0
                                ));
                                ui.label(format!("Type: {:?}", definition.province_type));
                                ui.label(format!("Coastal: {:?}", definition.coastal.0));
                                ui.label(format!("Terrain: {:?}", definition.terrain.0));
                                continent.map(|c| ui.label(format!("Continent: {:?}", c.0)));
                            }
                        }
                        MapDisplayMode::Rivers => {}
                        m => {
                            ui.label(format!("Unknown map mode: {m}"));
                        }
                    });
                TopBottomPanel::bottom("log_panel")
                    .max_height(200.0)
                    .show_inside(ui, |ui| {
                        ui.label("Log Panel");
                        ui.set_style(egui::Style {
                            wrap: Some(false),
                            ..Default::default()
                        });
                        ui.label(self.terminal.contents());
                    });
            });
        Ok(())
    }
}
