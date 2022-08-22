use crate::ui::map_loader::GetMap;
use crate::ui::map_mode::GetMapMode;
use crate::ui::selection::{
    GetSelectedPoint, GetSelectedProvince, GetSelectedState, GetSelectedStrategicRegion, Selection,
    SetSelectedProvince, SetSelectedState, SetSelectedStrategicRegion,
};
use crate::{MapError, MapLoader, MapMode};
use actix::Addr;
use egui::{Context, Pos2, SidePanel, TopBottomPanel, Ui};
use indicatif::InMemoryTerm;
use log::{debug, trace};
use std::fmt::Display;
use world_gen::components::prelude::{Definition, StrategicRegion};
use world_gen::components::state::State;
use world_gen::components::wrappers::Continent;
use world_gen::map::{
    GetContinentFromIndex, GetProvinceDefinitionFromId, GetProvinceIdFromPoint, GetStateFromId,
    GetStateIdFromPoint, GetStrategicRegionFromId, GetStrategicRegionIdFromPoint, Map,
};
use world_gen::MapDisplayMode;

struct SelectedRegions {
    selected_strategic_region: Option<StrategicRegion>,
    selected_state: Option<State>,
    selected_province: Option<Definition>,
    selected_point: Option<Pos2>,
}

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
        let selected_regions = self.get_selected_regions().await?;
        self.update_selected_regions(map_mode, &map_addr, &selected_regions)
            .await?;
        let continent_index = selected_regions
            .selected_province
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
                render_info_panel(map_mode, &map_addr, &selected_regions, continent, ui);
                self.render_log_panel(ui);
            });
        Ok(())
    }

    async fn update_selected_regions(
        &self,
        map_mode: MapDisplayMode,
        map_addr: &Option<Addr<Map>>,
        selected_regions: &SelectedRegions,
    ) -> Result<(), MapError> {
        if let (Some(map), Some(point)) = (map_addr.clone(), selected_regions.selected_point) {
            match map_mode {
                MapDisplayMode::HeightMap | MapDisplayMode::Terrain | MapDisplayMode::Rivers => {}
                MapDisplayMode::Provinces => {
                    if selected_regions.selected_province.is_none() {
                        if let Some(province_id) =
                            map.send(GetProvinceIdFromPoint::new(point)).await?
                        {
                            if let Some(def) = map
                                .send(GetProvinceDefinitionFromId::new(province_id))
                                .await?
                            {
                                self.selection.send(SetSelectedProvince::new(def)).await?;
                            }
                        }
                    }
                }
                MapDisplayMode::StrategicRegions => {
                    if selected_regions.selected_strategic_region.is_none() {
                        if let Some(sr_id) =
                            map.send(GetStrategicRegionIdFromPoint::new(point)).await?
                        {
                            if let Some(sr) = map.send(GetStrategicRegionFromId::new(sr_id)).await?
                            {
                                self.selection
                                    .send(SetSelectedStrategicRegion::new(sr))
                                    .await?;
                            }
                        }
                    }
                }
                MapDisplayMode::States => {
                    if selected_regions.selected_state.is_none() {
                        if let Some(s_id) = map.send(GetStateIdFromPoint::new(point)).await? {
                            if let Some(s) = map.send(GetStateFromId::new(s_id)).await? {
                                self.selection.send(SetSelectedState::new(s)).await?;
                            }
                        }
                    }
                }
                m => {}
            }
        }

        Ok(())
    }

    async fn get_selected_regions(&self) -> Result<SelectedRegions, MapError> {
        let selected_point: Option<Pos2> = self.selection.send(GetSelectedPoint).await?;
        let selected_province: Option<Definition> =
            self.selection.send(GetSelectedProvince).await?;
        let selected_state: Option<State> = self.selection.send(GetSelectedState).await?;
        let selected_strategic_region: Option<StrategicRegion> =
            self.selection.send(GetSelectedStrategicRegion).await?;
        let selected_regions = SelectedRegions {
            selected_strategic_region,
            selected_state,
            selected_province,
            selected_point,
        };
        Ok(selected_regions)
    }

    fn render_log_panel(&self, ui: &mut Ui) {
        TopBottomPanel::bottom("log_panel")
            .max_height(200.0)
            .show_inside(ui, |ui| {
                ui.heading("Log Panel");
                ui.set_style(egui::Style {
                    wrap: Some(false),
                    ..Default::default()
                });
                ui.label(self.terminal.contents());
            });
    }
}

fn render_info_panel(
    map_mode: MapDisplayMode,
    map_addr: &Option<Addr<Map>>,
    selected_regions: &SelectedRegions,
    continent: Option<Continent>,
    ui: &mut Ui,
) {
    TopBottomPanel::top("info_panel")
        .min_height(200.0)
        .max_height(600.0)
        .show_inside(ui, |ui| match map_mode {
            MapDisplayMode::Provinces => {
                render_province_info(map_addr, selected_regions, continent, ui);
            }
            MapDisplayMode::States => {
                render_state_info(map_addr, selected_regions, ui);
            }
            MapDisplayMode::StrategicRegions => {
                render_strategic_region_info(map_addr, selected_regions, ui);
            }
            MapDisplayMode::HeightMap | MapDisplayMode::Terrain | MapDisplayMode::Rivers => {}
            m => {
                ui.label(format!("Unknown map mode: {m}"));
            }
        });
}

fn render_strategic_region_info(
    map_addr: &Option<Addr<Map>>,
    selected_regions: &SelectedRegions,
    ui: &mut Ui,
) {
    ui.heading("Strategic Region Information");
    if let (Some(_), Some(_), Some(sr)) = (
        map_addr,
        selected_regions.selected_point,
        &selected_regions.selected_strategic_region,
    ) {
        ui.label(format!("Id: {:?}", sr.id.0));
        ui.label(format!("Name: {:?}", sr.name.0));
        list_items(ui, &sr.provinces.iter().collect::<Vec<_>>());
    }
}

fn render_state_info(
    map_addr: &Option<Addr<Map>>,
    selected_regions: &SelectedRegions,
    ui: &mut Ui,
) {
    ui.heading("State Information");
    if let (Some(_), Some(_), Some(state)) = (
        map_addr,
        selected_regions.selected_point,
        &selected_regions.selected_state,
    ) {
        ui.label(format!("Id: {:?}", state.id.0));
        ui.label(format!("Name: {:?}", state.name.0));
        ui.label(format!(
            "Manpower: {:?}",
            state.manpower[state.manpower.len() - 1].0
        ));
        if let Some(supplies) = state.local_supplies {
            ui.label(format!("Local Supplies: {:?}", supplies.0));
        }
        if let Some(max_level_factor) = state.buildings_max_level_factor {
            ui.label(format!(
                "Buildings Max Level Factor: {:?}",
                max_level_factor.0
            ));
        }
        if let Some(impassable) = state.impassable {
            ui.label(format!("Impassable: {:?}", impassable));
        }
        ui.label(format!(
            "Category: {:?}",
            state.state_category[state.state_category.len() - 1].0
        ));
        if let Some(history) = &state.history {
            ui.collapsing("History", |ui| {
                ui.label(format!("Owner: {:?}", history.owner.0));
                if let Some(controller) = &history.controller {
                    ui.label(format!("Controller: {:?}", controller.0));
                }
                ui.collapsing("Victory Points", |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([true, true])
                        .show(ui, |ui| {
                            for (id, vp) in &history.victory_points {
                                ui.label(format!("{:?}: {:?}", id.0, vp.0));
                            }
                        });
                });
            });
        }
        list_items(ui, &state.provinces.iter().collect::<Vec<_>>());
    }
}

fn list_items<T: Display>(ui: &mut Ui, list: &[T]) {
    ui.collapsing("Provinces", |ui| {
        egui::ScrollArea::vertical()
            .auto_shrink([true, true])
            .show(ui, |ui| {
                for item in list {
                    ui.label(format!("{}", item));
                }
            });
    });
}

fn render_province_info(
    map_addr: &Option<Addr<Map>>,
    selected_regions: &SelectedRegions,
    continent: Option<Continent>,
    ui: &mut Ui,
) {
    ui.heading("Province Information");
    if let (Some(_), Some(_), Some(definition)) = (
        map_addr,
        selected_regions.selected_point,
        &selected_regions.selected_province,
    ) {
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
