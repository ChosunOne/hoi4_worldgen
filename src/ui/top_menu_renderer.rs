use crate::{RootPath, SetRootPath};
use actix::{Actor, Addr, Context as ActixContext, Handler, Message, ResponseFuture};
use egui::menu::bar;
use egui::{Context, TopBottomPanel};
use log::{debug, error};
use world_gen::MapError;

/// A request to render the top menu bar.
#[derive(Message)]
#[rtype(result = "Result<(), MapError>")]
#[non_exhaustive]
pub struct RenderTopMenuBar {
    pub context: Context,
}

impl RenderTopMenuBar {
    pub const fn new(context: Context) -> Self {
        Self { context }
    }
}

pub struct TopMenuRenderer {
    root_path: Addr<RootPath>,
}

impl TopMenuRenderer {
    #[inline]
    pub const fn new(root_path: Addr<RootPath>) -> Self {
        Self { root_path }
    }
}

impl Actor for TopMenuRenderer {
    type Context = ActixContext<Self>;
}

impl Handler<RenderTopMenuBar> for TopMenuRenderer {
    type Result = ResponseFuture<Result<(), MapError>>;

    fn handle(&mut self, msg: RenderTopMenuBar, _ctx: &mut Self::Context) -> Self::Result {
        debug!("RenderTopMenuBar");
        let context = msg.context;
        let root_path = self.root_path.clone();
        Box::pin(async move {
            TopBottomPanel::top("top_panel").show(&context, |ui| {
                bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Open root folder").clicked() {
                            if let Err(e) = root_path.try_send(SetRootPath) {
                                error!("{e}");
                            }
                        }
                    })
                });
            });
            Ok(())
        })
    }
}
