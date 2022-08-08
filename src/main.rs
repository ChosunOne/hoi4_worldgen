#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::MapLoadingState::{Loading, ScheduledForLoad};
use eframe::egui::{menu::bar, CentralPanel, ColorImage, TextureHandle, TopBottomPanel, Ui};
use eframe::App;
use image::{DynamicImage, RgbImage};
use std::path::PathBuf;
use world_gen::map::Map;

#[derive(Default, Clone)]
struct MapImages {
    heightmap_image: Option<ColorImage>,
    terrain_image: Option<ColorImage>,
    provinces_image: Option<ColorImage>,
    rivers_image: Option<ColorImage>,
}

#[derive(Default, Clone)]
struct MapTextures {
    heightmap_texture: Option<TextureHandle>,
    terrain_texture: Option<TextureHandle>,
    provinces_texture: Option<TextureHandle>,
    rivers_texture: Option<TextureHandle>,
}

#[derive(Default, Clone)]
struct WorldGenApp {
    root_path: Option<PathBuf>,
    map: Option<Map>,
    map_err_text: Option<String>,
    map_loading_state: MapLoadingState,
    map_display_mode: MapDisplayMode,
    images: MapImages,
    textures: MapTextures,
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
    fn load_map(&mut self, ui: &mut Ui) {
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

        self.set_map_mode(MapDisplayMode::HeightMap);
        self.set_map_mode(MapDisplayMode::Terrain);
        self.set_map_mode(MapDisplayMode::Provinces);
        self.set_map_mode(MapDisplayMode::Rivers);
        self.set_map_mode(MapDisplayMode::HeightMap);

        Self::render_map_mode(
            ui,
            &mut self.images.heightmap_image,
            &mut self.textures.heightmap_texture,
        );
        Self::render_map_mode(
            ui,
            &mut self.images.terrain_image,
            &mut self.textures.terrain_texture,
        );
        Self::render_map_mode(
            ui,
            &mut self.images.provinces_image,
            &mut self.textures.provinces_texture,
        );
        Self::render_map_mode(
            ui,
            &mut self.images.rivers_image,
            &mut self.textures.rivers_texture,
        );

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
                    if self.textures.heightmap_texture.is_none() {
                        let image = self.load_map_image(&map.heightmap);
                        self.images.heightmap_image = Some(image);
                    }
                }
                MapDisplayMode::Terrain => {
                    if self.textures.terrain_texture.is_none() {
                        let image = self.load_map_image(&map.terrain);
                        self.images.terrain_image = Some(image);
                    }
                }
                MapDisplayMode::Provinces => {
                    if self.textures.provinces_texture.is_none() {
                        let image = self.load_map_image(&map.provinces);
                        self.images.provinces_image = Some(image);
                    }
                }
                MapDisplayMode::Rivers => {
                    if self.textures.rivers_texture.is_none() {
                        let image = self.load_map_image(&map.rivers);
                        self.images.rivers_image = Some(image);
                    }
                }
            };
        }
    }

    fn set_map_mode(&mut self, mode: MapDisplayMode) {
        self.map_display_mode = mode;
        self.load_map_display();
    }

    fn clear_map(&mut self) {
        self.root_path = None;
        self.images = MapImages::default();
        self.textures = MapTextures::default();
        self.map = None;
    }

    fn render_map_mode(
        ui: &mut Ui,
        image: &mut Option<ColorImage>,
        texture: &mut Option<TextureHandle>,
    ) {
        if let Some(image) = image.take() {
            *texture = Some(ui.ctx().load_texture("map", image));
        }
        if let Some(tex) = &texture {
            ui.image(tex, tex.size_vec2());
        }
    }
}

impl App for WorldGenApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open root folder").clicked() {
                        self.clear_map();
                        self.root_path = rfd::FileDialog::new().pick_folder();
                    }
                })
            })
        });
        CentralPanel::default().show(ctx, |ui| {
            if self.map_loading_state == ScheduledForLoad {
                self.map_loading_state = Loading;
            }
            if self.root_path.is_none() {
                ui.heading("Please select a root folder");
            }
            if let Some(path) = self.root_path.as_ref().map(|p| p.display().to_string()) {
                ui.horizontal(|ui| {
                    ui.label("Root Directory: ");
                    ui.label(path);
                    if self.map.is_none() && ui.button("Load Map").clicked() {
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
                self.load_map(ui);
                self.load_map_display();
            }
            if self.map.is_some() {
                ui.horizontal(|ui| {
                    if ui.button("Height Map").clicked() {
                        self.set_map_mode(MapDisplayMode::HeightMap);
                    }
                    if ui.button("Terrain").clicked() {
                        self.set_map_mode(MapDisplayMode::Terrain);
                    }
                    if ui.button("Provinces").clicked() {
                        self.set_map_mode(MapDisplayMode::Provinces);
                    }
                    if ui.button("Rivers").clicked() {
                        self.set_map_mode(MapDisplayMode::Rivers);
                    }
                });
            }
            match self.map_display_mode {
                MapDisplayMode::HeightMap => {
                    Self::render_map_mode(
                        ui,
                        &mut self.images.heightmap_image,
                        &mut self.textures.heightmap_texture,
                    );
                }
                MapDisplayMode::Terrain => {
                    Self::render_map_mode(
                        ui,
                        &mut self.images.terrain_image,
                        &mut self.textures.terrain_texture,
                    );
                }
                MapDisplayMode::Provinces => {
                    Self::render_map_mode(
                        ui,
                        &mut self.images.provinces_image,
                        &mut self.textures.provinces_texture,
                    );
                }
                MapDisplayMode::Rivers => {
                    Self::render_map_mode(
                        ui,
                        &mut self.images.rivers_image,
                        &mut self.textures.rivers_texture,
                    );
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
