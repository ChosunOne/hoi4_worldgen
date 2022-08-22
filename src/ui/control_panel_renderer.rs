use crate::ui::map_loader::{GetMap, IsMapLoading, LoadMap, MapLoader};
use crate::ui::map_mode::SetMapMode;
use crate::ui::root_path::GetRootPath;
use crate::{MapError, MapMode, RootPath};
use actix::Addr;
use egui::{Context, TopBottomPanel};
use indicatif::InMemoryTerm;
use log::{debug, error, trace};
use std::path::PathBuf;
use world_gen::MapDisplayMode;

pub struct ControlPanelRenderer {
    root_path: Addr<RootPath>,
    map_loader: Addr<MapLoader>,
    map_mode: Addr<MapMode>,
    terminal: InMemoryTerm,
}

impl ControlPanelRenderer {
    #[inline]
    pub const fn new(
        root_path: Addr<RootPath>,
        map_loader: Addr<MapLoader>,
        map_mode: Addr<MapMode>,
        terminal: InMemoryTerm,
    ) -> Self {
        Self {
            root_path,
            map_loader,
            map_mode,
            terminal,
        }
    }

    pub async fn render_control_panel(&self, ctx: &Context) -> Result<(), MapError> {
        let root_path: Option<PathBuf> = self.root_path.send(GetRootPath).await?;
        let map = self.map_loader.send(GetMap).await?;
        let is_map_loading = self.map_loader.send(IsMapLoading).await?;
        TopBottomPanel::top("control_panel").show(ctx, |ui| {
            if let Some(pathbuf) = root_path {
                ui.horizontal(|ui| {
                    ui.label("Root Directory: ");
                    ui.label(pathbuf.display().to_string());
                    if map.is_none() && ui.button("Load Map").clicked() {
                        if let Err(e) = self
                            .map_loader
                            .try_send(LoadMap::new(pathbuf, self.terminal.clone()))
                        {
                            error!("{e}");
                        }
                    }
                });
                if is_map_loading {
                    ui.spinner();
                }
            } else {
                ui.heading("Please select a root folder");
            }
            if map.is_some() {
                ui.horizontal(|ui| {
                    if ui.button("Height Map").clicked() {
                        self.map_mode
                            .do_send(SetMapMode::new(MapDisplayMode::HeightMap));
                    }
                    if ui.button("Terrain").clicked() {
                        self.map_mode
                            .do_send(SetMapMode::new(MapDisplayMode::Terrain));
                    }
                    if ui.button("Rivers").clicked() {
                        self.map_mode
                            .do_send(SetMapMode::new(MapDisplayMode::Rivers));
                    }
                    if ui.button("Provinces").clicked() {
                        self.map_mode
                            .do_send(SetMapMode::new(MapDisplayMode::Provinces));
                    }
                    if ui.button("States").clicked() {
                        self.map_mode
                            .do_send(SetMapMode::new(MapDisplayMode::States));
                    }
                    if ui.button("Strategic Regions").clicked() {
                        self.map_mode
                            .do_send(SetMapMode::new(MapDisplayMode::StrategicRegions));
                    }
                });
            }
        });
        Ok(())
    }
}
