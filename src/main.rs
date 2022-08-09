#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{menu::bar, CentralPanel, ColorImage, TextureHandle, TopBottomPanel, Ui};
use eframe::App;
use egui::Vec2;
use image::{DynamicImage, RgbImage};
use log::error;
use std::path::PathBuf;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::task::JoinHandle;
use world_gen::map::Map;

#[derive(Default)]
struct MapImages {
    heightmap_image: Option<ColorImage>,
    heightmap_image_receiver: Option<Receiver<ColorImage>>,
    heightmap_image_handle: Option<JoinHandle<()>>,
    terrain_image: Option<ColorImage>,
    terrain_image_receiver: Option<Receiver<ColorImage>>,
    terrain_image_handle: Option<JoinHandle<()>>,
    provinces_image: Option<ColorImage>,
    provinces_image_receiver: Option<Receiver<ColorImage>>,
    provinces_image_handle: Option<JoinHandle<()>>,
    rivers_image: Option<ColorImage>,
    rivers_image_receiver: Option<Receiver<ColorImage>>,
    rivers_image_handle: Option<JoinHandle<()>>,
}

#[derive(Default)]
struct MapTextures {
    heightmap_texture: Option<TextureHandle>,
    terrain_texture: Option<TextureHandle>,
    provinces_texture: Option<TextureHandle>,
    rivers_texture: Option<TextureHandle>,
}

#[derive(Default)]
struct WorldGenApp {
    root_path: Option<PathBuf>,
    map_handle: Option<JoinHandle<()>>,
    map_receiver: Option<Receiver<Map>>,
    map: Option<Map>,
    map_err_text: Option<String>,
    map_display_mode: MapDisplayMode,
    images: MapImages,
    textures: MapTextures,
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
    fn create_map_textures(&mut self, ui: &mut Ui) {
        self.set_map_mode(MapDisplayMode::HeightMap);
        self.set_map_mode(MapDisplayMode::Terrain);
        self.set_map_mode(MapDisplayMode::Provinces);
        self.set_map_mode(MapDisplayMode::Rivers);
        self.set_map_mode(MapDisplayMode::HeightMap);

        Self::render_map(
            ui,
            &mut self.images.heightmap_image,
            &mut self.textures.heightmap_texture,
        );
        Self::render_map(
            ui,
            &mut self.images.terrain_image,
            &mut self.textures.terrain_texture,
        );
        Self::render_map(
            ui,
            &mut self.images.provinces_image,
            &mut self.textures.provinces_texture,
        );
        Self::render_map(
            ui,
            &mut self.images.rivers_image,
            &mut self.textures.rivers_texture,
        );
    }

    fn load_map_image(image: RgbImage) -> ColorImage {
        let size = [image.width() as _, image.height() as _];
        let image_buffer = DynamicImage::ImageRgb8(image).into_rgba8();
        let pixels = image_buffer.as_flat_samples();
        ColorImage::from_rgba_unmultiplied(size, pixels.as_slice())
    }

    fn load_map_display(&mut self) {
        if let Some(map) = &self.map {
            match self.map_display_mode {
                MapDisplayMode::HeightMap => Self::load_image(
                    &map.heightmap,
                    &mut self.textures.heightmap_texture,
                    &mut self.images.heightmap_image_receiver,
                    &mut self.images.heightmap_image_handle,
                ),
                MapDisplayMode::Terrain => Self::load_image(
                    &map.terrain,
                    &mut self.textures.terrain_texture,
                    &mut self.images.terrain_image_receiver,
                    &mut self.images.terrain_image_handle,
                ),
                MapDisplayMode::Provinces => Self::load_image(
                    &map.provinces,
                    &mut self.textures.provinces_texture,
                    &mut self.images.provinces_image_receiver,
                    &mut self.images.provinces_image_handle,
                ),
                MapDisplayMode::Rivers => Self::load_image(
                    &map.rivers,
                    &mut self.textures.rivers_texture,
                    &mut self.images.rivers_image_receiver,
                    &mut self.images.rivers_image_handle,
                ),
            };
        }
    }

    fn load_image(
        map_image: &RgbImage,
        texture: &mut Option<TextureHandle>,
        receiver: &mut Option<Receiver<ColorImage>>,
        handle: &mut Option<JoinHandle<()>>,
    ) {
        if texture.is_none() {
            let (tx, rx) = channel(1);
            *receiver = Some(rx);
            let image = map_image.clone();
            let h = tokio::spawn(async move {
                if let Err(e) = tx.send(Self::load_map_image(image)).await {
                    error!("{}", e);
                }
            });
            *handle = Some(h);
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

    fn render_map(
        ui: &mut Ui,
        image: &mut Option<ColorImage>,
        texture: &mut Option<TextureHandle>,
    ) {
        if let Some(image) = image.take() {
            *texture = Some(ui.ctx().load_texture("map", image));
        }
        if let Some(tex) = &texture {
            let size = ui.ctx().available_rect().size() * 0.8;
            let tex_size = tex.size_vec2();
            let x_scale = size.x / tex_size.x;
            let y_scale = size.y / tex_size.y;
            let min_scale = x_scale.min(y_scale);
            ui.image(tex, tex_size * min_scale);
        }
    }

    fn update_item<T>(
        receiver: &mut Option<Receiver<T>>,
        item: &mut Option<T>,
        handle: &mut Option<JoinHandle<()>>,
    ) {
        if let Some(r) = receiver {
            if let Ok(thing) = r.try_recv() {
                *item = Some(thing);
                handle.take();
                receiver.take();
            }
        }
    }

    fn update_images(images: &mut MapImages) {
        Self::update_item(
            &mut images.heightmap_image_receiver,
            &mut images.heightmap_image,
            &mut images.heightmap_image_handle,
        );
        Self::update_item(
            &mut images.terrain_image_receiver,
            &mut images.terrain_image,
            &mut images.terrain_image_handle,
        );
        Self::update_item(
            &mut images.provinces_image_receiver,
            &mut images.provinces_image,
            &mut images.provinces_image_handle,
        );
        Self::update_item(
            &mut images.rivers_image_receiver,
            &mut images.rivers_image,
            &mut images.rivers_image_handle,
        );
    }

    fn load_map_button(&mut self, ui: &mut Ui) {
        if self.map.is_none() {
            if ui.button("Load Map").clicked() {
                let (tx, rx) = channel(1);
                let path = self.root_path.clone().unwrap();
                self.map_receiver = Some(rx);
                self.map_handle = Some(tokio::spawn(async move {
                    match Map::new(&path) {
                        Ok(m) => {
                            if let Err(e) = tx.send(m).await {
                                error!("{}", e);
                            }
                        }
                        Err(e) => {
                            error!("{:?}", e);
                        }
                    }
                }));
            }
            if let Some(map_err_text) = &self.map_err_text {
                ui.label(map_err_text);
            }
            self.create_map_textures(ui);
        }
    }
}

impl App for WorldGenApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        Self::update_item(&mut self.map_receiver, &mut self.map, &mut self.map_handle);
        Self::update_images(&mut self.images);

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
            if self.root_path.is_none() {
                ui.heading("Please select a root folder");
            }
            if let Some(path) = self.root_path.as_ref().map(|p| p.display().to_string()) {
                ui.horizontal(|ui| {
                    ui.label("Root Directory: ");
                    ui.label(path);
                    self.load_map_button(ui);
                });
            }
            if self.map_handle.is_some() {
                ui.label("Loading map...");
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
                    if self.images.heightmap_image_handle.is_some() {
                        ui.label("Loading heightmap...");
                    }
                    if self.images.terrain_image_handle.is_some() {
                        ui.label("Loading terrain...");
                    }
                    if self.images.provinces_image_handle.is_some() {
                        ui.label("Loading provinces...");
                    }
                    if self.images.rivers_image_handle.is_some() {
                        ui.label("Loading rivers...");
                    }
                });
            }
            match self.map_display_mode {
                MapDisplayMode::HeightMap => {
                    Self::render_map(
                        ui,
                        &mut self.images.heightmap_image,
                        &mut self.textures.heightmap_texture,
                    );
                }
                MapDisplayMode::Terrain => {
                    Self::render_map(
                        ui,
                        &mut self.images.terrain_image,
                        &mut self.textures.terrain_texture,
                    );
                }
                MapDisplayMode::Provinces => {
                    Self::render_map(
                        ui,
                        &mut self.images.provinces_image,
                        &mut self.textures.provinces_texture,
                    );
                }
                MapDisplayMode::Rivers => {
                    Self::render_map(
                        ui,
                        &mut self.images.rivers_image,
                        &mut self.textures.rivers_texture,
                    );
                }
            }
        });
    }
}

#[tokio::main]
async fn main() {
    use std::default::Default;
    let options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Hearts of Iron IV Map Editor",
        options,
        Box::new(|_cc| Box::new(WorldGenApp::default())),
    );
}
