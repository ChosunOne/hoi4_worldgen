use crate::ui::map_loader::GetMap;
use crate::ui::map_mode::GetMapMode;
use crate::ui::selection::{GetSelectedPoint, GetSelectedProvince, Selection, SetSelectedProvince};
use crate::{MapError, MapLoader, MapMode};
use actix::{Actor, Addr, Context as ActixContext, Handler, Message, ResponseFuture};
use egui::{Context, Pos2, SidePanel, TopBottomPanel};
use indicatif::InMemoryTerm;
use log::debug;
use world_gen::components::prelude::{Definition, ProvinceId};
use world_gen::components::wrappers::Continent;
use world_gen::map::{
    GetContinentFromIndex, GetProvinceDefinitionFromId, GetProvinceIdFromPoint, Map,
};
use world_gen::MapDisplayMode;

/// A request to render the right panel
#[derive(Message)]
#[rtype(result = "Result<(), MapError>")]
#[non_exhaustive]
pub struct RenderRightPanel {
    pub context: Context,
}

impl RenderRightPanel {
    #[inline]
    pub const fn new(context: Context) -> Self {
        Self { context }
    }
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
}

impl Actor for RightPanelRenderer {
    type Context = ActixContext<Self>;
}

impl Handler<RenderRightPanel> for RightPanelRenderer {
    type Result = ResponseFuture<Result<(), MapError>>;

    fn handle(&mut self, msg: RenderRightPanel, _ctx: &mut Self::Context) -> Self::Result {
        debug!("RenderRightPanel");
        let context = msg.context;
        let map_mode_addr = self.map_mode.clone();
        let map_loader_addr = self.map_loader.clone();
        let selection_addr = self.selection.clone();
        let terminal = self.terminal.clone();
        Box::pin(async move {
            let map_mode: MapDisplayMode = map_mode_addr.send(GetMapMode).await?;
            let map_addr: Option<Addr<Map>> = map_loader_addr.send(GetMap).await?;
            let selected_point: Option<Pos2> = selection_addr.send(GetSelectedPoint).await?;
            let selected_province: Option<Definition> =
                selection_addr.send(GetSelectedProvince).await?;
            if let (Some(p), None, Some(m)) =
                (selected_point, selected_province.clone(), map_addr.clone())
            {
                // TODO: Perhaps reconsider where this logic should live
                if let Some(province_id) = m.send(GetProvinceIdFromPoint::new(p)).await? {
                    if let Some(def) = m
                        .send(GetProvinceDefinitionFromId::new(province_id))
                        .await?
                    {
                        selection_addr.send(SetSelectedProvince::new(def)).await?;
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
                .show(&context, |ui| {
                    TopBottomPanel::top("info_panel")
                        .min_height(200.0)
                        .show_inside(ui, |ui| match map_mode {
                            MapDisplayMode::HeightMap => {}
                            MapDisplayMode::Terrain => {}
                            MapDisplayMode::Provinces => {
                                ui.label("Province Information");
                                if let (Some(map), Some(point), Some(definition)) =
                                    (map_addr, selected_point, selected_province)
                                {
                                    ui.label(format!("Id: {:?}", definition.id.0));
                                    ui.label(format!(
                                        "Color: ({:?}, {:?}, {:?}",
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
                            ui.label(terminal.contents());
                        });
                });
            Ok(())
        })
    }
}
