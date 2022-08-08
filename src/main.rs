#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::MapLoadingState::{Loading, ScheduledForLoad};
use eframe::egui;
use eframe::App;
use egui::{ColorImage, TextureHandle};
use image::{DynamicImage, RgbImage};
use std::path::PathBuf;
use world_gen::map::Map;

#[derive(Default)]
struct WorldGenApp {
    root_path: Option<PathBuf>,
    map: Option<Map>,
    map_err_text: Option<String>,
    map_loading_state: MapLoadingState,
    map_display_mode: MapDisplayMode,
    heightmap_image: Option<ColorImage>,
    terrain_image: Option<ColorImage>,
    province_image: Option<ColorImage>,
    rivers_image: Option<ColorImage>,
    heightmap_texture: Option<TextureHandle>,
    terrain_texture: Option<TextureHandle>,
    provinces_texture: Option<TextureHandle>,
    rivers_texture: Option<TextureHandle>,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
enum MapLoadingState {
    #[default]
    NotLoading,
    ScheduledForLoad,
    Loading,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
enum MapDisplayMode {
    #[default]
    HeightMap,
    Terrain,
    Provinces,
    Rivers,
}

impl WorldGenApp {
    fn load_map(&mut self) {
        if let Some(path) = &self.root_path {
            match Map::new(path) {
                Ok(map) => {
                    self.map = Some(map);
                    self.map_err_text = None;
                }
                Err(err) => {
                    self.map_err_text = Some(err.to_string());
                }
            }
        }

        self.map_loading_state = MapLoadingState::NotLoading;
    }

    fn load_map_image(&self, image: &RgbImage) -> ColorImage {
        let size = [image.width() as _, image.height() as _];
        let image_buffer = DynamicImage::ImageRgb8(image.clone()).into_rgba8();
        let pixels = image_buffer.as_flat_samples();
        ColorImage::from_rgba_unmultiplied(size, pixels.as_slice())
    }

    fn load_map_display(&mut self) {
        if let Some(map) = &self.map {
            match self.map_display_mode {
                MapDisplayMode::HeightMap => {
                    if self.heightmap_texture.is_none() {
                        let image = self.load_map_image(&map.heightmap);
                        self.heightmap_image = Some(image);
                    }
                }
                MapDisplayMode::Terrain => {
                    if self.terrain_texture.is_none() {
                        let image = self.load_map_image(&map.terrain);
                        self.terrain_image = Some(image);
                    }
                }
                MapDisplayMode::Provinces => {
                    if self.provinces_texture.is_none() {
                        let image = self.load_map_image(&map.provinces);
                        self.province_image = Some(image);
                    }
                }
                MapDisplayMode::Rivers => {
                    if self.rivers_texture.is_none() {
                        let image = self.load_map_image(&map.rivers);
                        self.rivers_image = Some(image);
                    }
                }
            };
        }
    }
}

impl App for WorldGenApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hearts of Iron IV Map Editor");
            if ui.button("Locate HOI4 Root Directory").clicked() {
                self.root_path = rfd::FileDialog::new().pick_folder();
            }
            if self.map_loading_state == ScheduledForLoad {
                self.map_loading_state = Loading;
            }
            if let Some(path) = &self.root_path {
                ui.horizontal(|ui| {
                    ui.label("Root Directory: ");
                    ui.label(path.display().to_string());
                    if ui.button("Load Map").clicked() {
                        self.map_loading_state = ScheduledForLoad;
                    }
                    if let Some(map_err_text) = &self.map_err_text {
                        ui.label(map_err_text);
                    }
                });
            }
            if self.map_loading_state == ScheduledForLoad {
                ui.label("Loading map...");
            }
            if self.map_loading_state == Loading {
                self.load_map();
                self.load_map_display();
            }
            if self.map.is_some() {
                ui.horizontal(|ui| {
                    if ui.button("Height Map").clicked() {
                        self.map_display_mode = MapDisplayMode::HeightMap;
                        self.load_map_display();
                    }
                    if ui.button("Terrain").clicked() {
                        self.map_display_mode = MapDisplayMode::Terrain;
                        self.load_map_display();
                    }
                    if ui.button("Provinces").clicked() {
                        self.map_display_mode = MapDisplayMode::Provinces;
                        self.load_map_display();
                    }
                    if ui.button("Rivers").clicked() {
                        self.map_display_mode = MapDisplayMode::Rivers;
                        self.load_map_display();
                    }
                });
            }
            match self.map_display_mode {
                MapDisplayMode::HeightMap => {
                    if let Some(image) = self.heightmap_image.take() {
                        self.heightmap_texture = Some(ui.ctx().load_texture("map", image));
                    }
                    if let Some(tex) = &self.heightmap_texture {
                        ui.image(tex, tex.size_vec2());
                    }
                }
                MapDisplayMode::Terrain => {
                    if let Some(image) = self.terrain_image.take() {
                        self.terrain_texture = Some(ui.ctx().load_texture("map", image));
                    }
                    if let Some(tex) = &self.terrain_texture {
                        ui.image(tex, tex.size_vec2());
                    }
                }
                MapDisplayMode::Provinces => {
                    if let Some(image) = self.province_image.take() {
                        self.provinces_texture = Some(ui.ctx().load_texture("map", image));
                    }
                    if let Some(tex) = &self.provinces_texture {
                        ui.image(tex, tex.size_vec2());
                    }
                }
                MapDisplayMode::Rivers => {
                    if let Some(image) = self.rivers_image.take() {
                        self.rivers_texture = Some(ui.ctx().load_texture("map", image));
                    }
                    if let Some(tex) = &self.rivers_texture {
                        ui.image(tex, tex.size_vec2());
                    }
                }
            }
        });
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Hearts of Iron IV Map Editor",
        options,
        Box::new(|_cc| Box::new(WorldGenApp::default())),
    );
}
