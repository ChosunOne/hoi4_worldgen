use crate::ui::central_panel_renderer::SetMap;
use crate::ui::map_loader::{GetMap, IsMapLoading, LoadMap, MapLoader};
use crate::ui::map_mode::{GetMapMode, SetMapMode};
use crate::ui::root_path::GetRootPath;
use crate::{MapError, MapMode, RootPath};
use actix::{Actor, Addr, Context as ActixContext, Handler, Message, ResponseFuture};
use egui::{Context, TopBottomPanel};
use indicatif::InMemoryTerm;
use log::{debug, error, trace};
use std::path::PathBuf;
use world_gen::MapDisplayMode;

/// A request to render the control panel
#[derive(Message)]
#[rtype(result = "Result<(), MapError>")]
#[non_exhaustive]
pub struct RenderControlPanel {
    pub context: Context,
}

impl RenderControlPanel {
    pub const fn new(context: Context) -> Self {
        Self { context }
    }
}

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
}

impl Actor for ControlPanelRenderer {
    type Context = ActixContext<Self>;
}

impl Handler<RenderControlPanel> for ControlPanelRenderer {
    type Result = ResponseFuture<Result<(), MapError>>;

    fn handle(&mut self, msg: RenderControlPanel, _ctx: &mut Self::Context) -> Self::Result {
        trace!("RenderControlPanel");
        let root_path_addr = self.root_path.clone();
        let map_loader_addr = self.map_loader.clone();
        let map_mode_addr = self.map_mode.clone();
        let terminal = self.terminal.clone();
        Box::pin(async move {
            let root_path: Option<PathBuf> = root_path_addr.send(GetRootPath).await?;
            let map = map_loader_addr.send(GetMap).await?;
            let is_map_loading = map_loader_addr.send(IsMapLoading).await?;
            TopBottomPanel::top("control_panel").show(&msg.context, |ui| {
                if let Some(pathbuf) = root_path {
                    ui.horizontal(|ui| {
                        ui.label("Root Directory: ");
                        ui.label(pathbuf.display().to_string());
                        if !map.is_some() && ui.button("Load Map").clicked() {
                            if let Err(e) =
                                map_loader_addr.try_send(LoadMap::new(pathbuf, terminal))
                            {
                                error!("{e}");
                            }
                        }
                    });
                    if is_map_loading {
                        ui.label("Loading map...");
                    }
                } else {
                    ui.heading("Please select a root folder");
                }
                if map.is_some() {
                    ui.horizontal(|ui| {
                        if ui.button("Height Map").clicked() {
                            map_mode_addr.do_send(SetMapMode::new(MapDisplayMode::HeightMap));
                        }
                        if ui.button("Terrain").clicked() {
                            map_mode_addr.do_send(SetMapMode::new(MapDisplayMode::Terrain));
                        }
                        if ui.button("Rivers").clicked() {
                            map_mode_addr.do_send(SetMapMode::new(MapDisplayMode::Rivers));
                        }
                        if ui.button("Provinces").clicked() {
                            map_mode_addr.do_send(SetMapMode::new(MapDisplayMode::Provinces));
                        }
                        if ui.button("States").clicked() {
                            map_mode_addr.do_send(SetMapMode::new(MapDisplayMode::States));
                        }
                        if ui.button("Strategic Regions").clicked() {
                            map_mode_addr
                                .do_send(SetMapMode::new(MapDisplayMode::StrategicRegions));
                        }
                    });
                }
            });

            Ok(())
        })
    }
}
