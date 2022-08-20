use crate::{RootPath, SetRootPath};
use actix::{Actor, Addr, Context as ActixContext, Handler, Message, ResponseFuture};
use egui::menu::bar;
use egui::{Context, TopBottomPanel};
use log::{debug, error, trace};
use world_gen::MapError;

pub struct TopMenuRenderer {
    root_path: Addr<RootPath>,
}

impl TopMenuRenderer {
    #[inline]
    pub const fn new(root_path: Addr<RootPath>) -> Self {
        Self { root_path }
    }

    pub fn render_top_menu_bar(&self, ctx: &Context) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open root folder").clicked() {
                        if let Err(e) = self.root_path.try_send(SetRootPath) {
                            error!("{e}");
                        }
                    }
                })
            });
        });
    }
}
