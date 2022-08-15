use crate::ui::map_mode::GetMapMode;
use crate::{MapError, MapMode};
use actix::{Actor, Addr, Context as ActixContext, Handler, Message, ResponseFuture};
use egui::{CentralPanel, Context};
use log::error;
use world_gen::map::Map;
use world_gen::MapDisplayMode;

/// A request to render the right panel
#[derive(Message)]
#[rtype(result = "Result<(), MapError>")]
#[non_exhaustive]
pub struct RenderCentralPanel {
    pub context: Context,
}

impl RenderCentralPanel {
    #[inline]
    pub const fn new(context: Context) -> Self {
        Self { context }
    }
}

/// A request to set the address of the map
#[derive(Message)]
#[rtype(result = "()")]
#[non_exhaustive]
pub struct SetMap(pub Addr<Map>);

#[derive(Debug)]
pub struct CentralPanelRenderer {
    map: Option<Addr<Map>>,
    map_mode: Addr<MapMode>,
}

impl CentralPanelRenderer {
    #[inline]
    pub const fn new(map_mode: Addr<MapMode>) -> Self {
        Self {
            map: None,
            map_mode,
        }
    }
}

impl Actor for CentralPanelRenderer {
    type Context = ActixContext<Self>;
}

impl Handler<RenderCentralPanel> for CentralPanelRenderer {
    type Result = ResponseFuture<Result<(), MapError>>;

    fn handle(&mut self, msg: RenderCentralPanel, ctx: &mut Self::Context) -> Self::Result {
        let context = msg.context;
        let map_mode_addr = self.map_mode.clone();
        Box::pin(async move {
            let map_mode: MapDisplayMode = map_mode_addr.send(GetMapMode).await?;
            CentralPanel::default().show(&context, |ui| match map_mode {
                MapDisplayMode::HeightMap => {}
                MapDisplayMode::Terrain => {}
                MapDisplayMode::Provinces => {}
                MapDisplayMode::Rivers => {}
                m => error!("Unknown MapDisplayMode: {m}"),
            });
            Ok(())
        })
    }
}

impl Handler<SetMap> for CentralPanelRenderer {
    type Result = ();

    fn handle(&mut self, msg: SetMap, ctx: &mut Self::Context) -> Self::Result {
        self.map = Some(msg.0);
    }
}
