use crate::ui::root_path::{GetRootPath, UpdateRootPath};
use crate::{RootPath, SetRootPath};
use actix::{Addr, Handler, Message, ResponseFuture};
use egui::menu::bar;
use egui::{Context, TopBottomPanel};
use log::{debug, error, trace};
use std::path::PathBuf;
use world_gen::MapError;

pub struct TopMenuRenderer {
    root_path: Addr<RootPath>,
    pub new_root_path: Option<PathBuf>,
    pub root_path_changed: bool,
}

impl TopMenuRenderer {
    #[inline]
    pub const fn new(root_path: Addr<RootPath>) -> Self {
        Self {
            root_path,
            new_root_path: None,
            root_path_changed: false,
        }
    }

    pub async fn render_top_menu_bar(&mut self, ctx: &Context) -> Result<(), MapError> {
        let root_path = self.root_path.send(GetRootPath).await?;
        if root_path.is_none() && self.new_root_path.is_some() {
            self.root_path
                .send(UpdateRootPath::new(self.new_root_path.clone()))
                .await?;
        }
        if root_path.is_some() && self.new_root_path.is_none() {
            debug!("Storing new root path");
            self.new_root_path = root_path.clone();
        }
        if root_path.is_some() && self.new_root_path.is_some() && self.new_root_path != root_path {
            debug!("Setting root path as changed");
            self.root_path_changed = true;
            self.new_root_path = root_path.clone();
        }

        let mut new_root_path = None;
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open root folder").clicked() {
                        new_root_path = Some(self.root_path.send(SetRootPath));
                        ui.close_menu();
                    }
                })
            });
        });

        if let Some(p) = new_root_path {
            debug!("New root path requested");
            p.await?;
        }

        Ok(())
    }
}
