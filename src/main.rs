#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{
    menu::bar, CentralPanel, ColorImage, SidePanel, TextureHandle, TopBottomPanel, Ui,
};
use eframe::App;
use egui::{ImageButton, Pos2, Rect, Sense, Vec2};
use image::{DynamicImage, RgbImage};
use indicatif::InMemoryTerm;
use log::error;
use std::mem::swap;
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

struct WorldGenApp {
    root_path: Option<PathBuf>,
    root_path_handle: Option<JoinHandle<()>>,
    root_path_receiver: Option<Receiver<PathBuf>>,
    map_handle: Option<JoinHandle<()>>,
    map_receiver: Option<Receiver<Map>>,
    map: Option<Map>,
    map_err_text: Option<String>,
    map_display_mode: MapDisplayMode,
    images: MapImages,
    textures: MapTextures,
    terminal: InMemoryTerm,
    map_region: Option<Rect>,
    zoom_level: Option<f32>,
}

impl Default for WorldGenApp {
    fn default() -> Self {
        Self {
            root_path: None,
            root_path_receiver: None,
            root_path_handle: None,
            map_handle: None,
            map_receiver: None,
            map: None,
            map_err_text: None,
            map_display_mode: MapDisplayMode::HeightMap,
            images: MapImages::default(),
            textures: MapTextures::default(),
            terminal: InMemoryTerm::new(16, 240),
            map_region: None,
            zoom_level: None,
        }
    }
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
    fn create_map_textures(&mut self) {
        if let Some(map) = &self.map {
            Self::load_image(
                &map.heightmap,
                &mut self.textures.heightmap_texture,
                &mut self.images.heightmap_image_receiver,
                &mut self.images.heightmap_image_handle,
            );
            Self::load_image(
                &map.terrain,
                &mut self.textures.terrain_texture,
                &mut self.images.terrain_image_receiver,
                &mut self.images.terrain_image_handle,
            );
            Self::load_image(
                &map.provinces,
                &mut self.textures.provinces_texture,
                &mut self.images.provinces_image_receiver,
                &mut self.images.provinces_image_handle,
            );
            Self::load_image(
                &map.rivers,
                &mut self.textures.rivers_texture,
                &mut self.images.rivers_image_receiver,
                &mut self.images.rivers_image_handle,
            );
        }
    }

    fn load_map_image(image: RgbImage) -> ColorImage {
        let size = [image.width() as _, image.height() as _];
        let image_buffer = DynamicImage::ImageRgb8(image).into_rgba8();
        let pixels = image_buffer.as_flat_samples();
        ColorImage::from_rgba_unmultiplied(size, pixels.as_slice())
    }

    fn load_image(
        map_image: &RgbImage,
        texture: &mut Option<TextureHandle>,
        receiver: &mut Option<Receiver<ColorImage>>,
        handle: &mut Option<JoinHandle<()>>,
    ) {
        if texture.is_none() && handle.is_none() {
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
    }

    fn clear_map(&mut self) {
        *self = WorldGenApp::default();
    }

    fn render_map(
        ui: &mut Ui,
        image: &mut Option<ColorImage>,
        texture: &mut Option<TextureHandle>,
        viewport: &Option<Rect>,
    ) {
        if let Some(image) = image.take() {
            *texture = Some(ui.ctx().load_texture("map", image));
        }
        if let Some(tex) = &texture {
            let mut viewport = viewport.map_or(
                Rect::from([Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)]),
                |r| r,
            );
            viewport.min.x = viewport.min.x.clamp(0.0, 1.0);
            viewport.min.y = viewport.min.y.clamp(0.0, 1.0);
            viewport.max.x = viewport.max.x.clamp(0.0, 1.0);
            viewport.max.y = viewport.max.y.clamp(0.0, 1.0);
            if viewport.min.x > viewport.max.x {
                swap(&mut viewport.min.x, &mut viewport.max.x);
            }
            if viewport.min.y > viewport.max.y {
                swap(&mut viewport.min.y, &mut viewport.max.y);
            }

            let size = ui.ctx().available_rect().size() * 0.8;
            let tex_size = tex.size_vec2();
            let x_scale = size.x / tex_size.x;
            let y_scale = size.y / tex_size.y;
            let min_scale = x_scale.min(y_scale);
            let image = ImageButton::new(tex, tex_size * min_scale)
                .uv(viewport)
                .sense(Sense {
                    click: true,
                    drag: true,
                    focusable: false,
                });
            let map = ui.add(image);
            let mouse_pos = ui.ctx().pointer_latest_pos();
            if let Some(pos) = mouse_pos {
                let map_rect = map.rect;
                if map_rect.contains(pos) {
                    let map_uv = Pos2::new(
                        (pos.x - map_rect.min.x).round(),
                        (pos.y - map_rect.min.y).round(),
                    );
                    ui.label(format!("({:?}, {:?})", map_uv.x, map_uv.y));
                }
            }
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
        if self.map.is_none() && self.map_handle.is_none() {
            if ui.button("Load Map").clicked() {
                let (tx, rx) = channel(1);
                let path = self.root_path.clone().unwrap();
                self.map_receiver = Some(rx);
                let terminal = self.terminal.clone();

                self.map_handle = Some(tokio::spawn(async move {
                    match Map::new(&path, &Some(terminal)).await {
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
        } else {
            self.create_map_textures();
        }
    }
}

impl App for WorldGenApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        Self::update_item(&mut self.map_receiver, &mut self.map, &mut self.map_handle);
        Self::update_item(
            &mut self.root_path_receiver,
            &mut self.root_path,
            &mut self.root_path_handle,
        );
        Self::update_images(&mut self.images);

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open root folder").clicked() {
                        self.clear_map();
                        let (tx, rx) = channel(1);
                        self.root_path_receiver = Some(rx);
                        self.root_path_handle = Some(tokio::spawn(async move {
                            if let Some(p) = rfd::FileDialog::new().pick_folder() {
                                if let Err(e) = tx.send(p).await {
                                    error!("{}", e);
                                }
                            }
                        }));
                    }
                })
            })
        });
        TopBottomPanel::top("control_panel").show(ctx, |ui| {
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
                });
            }
        });
        SidePanel::right("right_panel")
            .resizable(false)
            .min_width(200.0)
            .show(ctx, |ui| {
                TopBottomPanel::top("info_panel")
                    .min_height(200.0)
                    .show_inside(ui, |ui| {
                        ui.label("Info Panel");
                    });
                TopBottomPanel::bottom("log_panel")
                    .max_height(200.0)
                    .show_inside(ui, |ui| {
                        ui.label("Log Panel");
                        ui.set_style(egui::Style {
                            wrap: Some(false),
                            ..Default::default()
                        });
                        ui.label(self.terminal.contents());
                    });
                ctx.request_repaint();
            });

        CentralPanel::default().show(ctx, |ui| {
            match self.map_display_mode {
                MapDisplayMode::HeightMap => {
                    Self::render_map(
                        ui,
                        &mut self.images.heightmap_image,
                        &mut self.textures.heightmap_texture,
                        &self.map_region,
                    );
                }
                MapDisplayMode::Terrain => {
                    Self::render_map(
                        ui,
                        &mut self.images.terrain_image,
                        &mut self.textures.terrain_texture,
                        &self.map_region,
                    );
                }
                MapDisplayMode::Provinces => {
                    Self::render_map(
                        ui,
                        &mut self.images.provinces_image,
                        &mut self.textures.provinces_texture,
                        &self.map_region,
                    );
                }
                MapDisplayMode::Rivers => {
                    Self::render_map(
                        ui,
                        &mut self.images.rivers_image,
                        &mut self.textures.rivers_texture,
                        &self.map_region,
                    );
                }
            }
            ctx.request_repaint();
        });
    }
}

#[tokio::main]
async fn main() {
    use std::default::Default;
    env_logger::init();
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
