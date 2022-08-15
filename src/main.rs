#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! Map generator for Hearts of Iron IV by Paradox Interactive.
#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    rust_2018_idioms,
    missing_debug_implementations,
    missing_docs
)]
#![allow(clippy::module_inception)]
#![allow(clippy::implicit_return)]
#![allow(clippy::blanket_clippy_restriction_lints)]
#![allow(clippy::shadow_same)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cargo_common_metadata)]
#![allow(clippy::separated_literal_suffix)]
#![allow(clippy::float_arithmetic)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::use_self)]
#![allow(clippy::pattern_type_mismatch)]
#![allow(clippy::pub_use)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::expect_used)]

mod ui;

use crate::ui::central_panel_renderer::{CentralPanelRenderer, RenderCentralPanel};
use crate::ui::control_panel_renderer::{ControlPanelRenderer, RenderControlPanel};
use crate::ui::map_loader::MapLoader;
use crate::ui::map_mode::MapMode;
use crate::ui::right_panel_renderer::{RenderRightPanel, RightPanelRenderer};
use crate::ui::root_path::RootPath;
use crate::ui::selection::Selection;
use crate::ui::top_menu_renderer::{RenderTopMenuBar, TopMenuRenderer};
use crate::ui::{root_path::SetRootPath, UiRenderer};
use actix::{Actor, Addr, System, SystemRunner};
use eframe::egui::{ColorImage, TextureHandle, TopBottomPanel, Ui};
use eframe::epaint::Rgba;
use eframe::{App, Storage};
use egui::{Context, ImageButton, Pos2, Rect, Response, Sense, Vec2, Visuals};
use image::{DynamicImage, RgbImage};
use indicatif::InMemoryTerm;
use log::{debug, error, info};
use std::mem::swap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::task::JoinHandle;
use world_gen::components::prelude::*;
use world_gen::map::{
    GetContinentFromIndex, GetMapImage, GetProvinceDefinitionFromId, GetProvinceIdFromPoint, Map,
};
use world_gen::{MapDisplayMode, MapError};

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
    system: Option<System>,
    map: Option<Addr<Map>>,
    map_err_text: Option<String>,
    map_display_mode: MapDisplayMode,
    images: MapImages,
    textures: MapTextures,
    terminal: InMemoryTerm,
    viewport: Option<Rect>,
    zoom_level: Option<f32>,
    selected_point: Option<Pos2>,
    selected_province: Option<Definition>,
    ui_renderer: Option<UiRenderer>,
    runtime: Option<Runtime>,
    system_thread: Option<JoinHandle<Result<(), MapError>>>,
}

impl Default for WorldGenApp {
    fn default() -> Self {
        Self {
            map: None,
            map_err_text: None,
            map_display_mode: MapDisplayMode::HeightMap,
            images: MapImages::default(),
            textures: MapTextures::default(),
            terminal: InMemoryTerm::new(16, 240),
            viewport: None,
            zoom_level: None,
            selected_point: None,
            selected_province: None,
            ui_renderer: None,
            runtime: None,
            system_thread: None,
            system: None,
        }
    }
}

impl WorldGenApp {
    fn initialize_renderer(&mut self) -> Result<(), MapError> {
        if self.runtime.is_some() {
            return Ok(());
        }
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        let (tx, rx) = std::sync::mpsc::channel();
        let terminal = self.terminal.clone();
        let (system_tx, system_rx) = std::sync::mpsc::channel();
        let system_thread = rt.spawn_blocking(move || {
            debug!("Spawning system");
            let system = System::new();

            system.block_on(async {
                debug!("Starting root path");
                let root_path = RootPath::default().start();
                debug!("Starting top menu renderer");
                let top_menu_renderer = TopMenuRenderer::new(root_path.clone()).start();
                debug!("Starting map loader");
                let map_loader = MapLoader::default().start();
                debug!("Starting map mode");
                let map_mode = MapMode::default().start();
                debug!("Starting control panel renderer");
                let control_panel_renderer = ControlPanelRenderer::new(
                    root_path,
                    map_loader.clone(),
                    map_mode.clone(),
                    terminal.clone(),
                )
                .start();
                debug!("Starting selection");
                let selection = Selection::default().start();
                debug!("Starting right panel renderer");
                let right_panel_renderer =
                    RightPanelRenderer::new(map_mode.clone(), selection, map_loader, terminal)
                        .start();
                debug!("Starting central panel renderer");
                let central_panel_renderer = CentralPanelRenderer::new(map_mode.clone()).start();
                let ui_renderer = UiRenderer::new(
                    top_menu_renderer,
                    control_panel_renderer,
                    right_panel_renderer,
                    central_panel_renderer,
                    map_mode,
                );
                debug!("Sending Ui Renderer");
                tx.send(ui_renderer).unwrap();
            });

            system_tx.send(System::current()).unwrap();

            debug!("Running system");
            system.run()?;
            info!("System stopped");
            Ok(())
        });
        let renderer = rx.recv()?;
        let system = system_rx.recv()?;
        self.runtime = Some(rt);
        self.ui_renderer = Some(renderer);
        self.system_thread = Some(system_thread);
        self.system = Some(system);
        Ok(())
    }

    async fn create_map_textures(&mut self) {
        if let Some(map) = &self.map {
            if let Ok(Some(heightmap)) = map.send(GetMapImage::new(MapDisplayMode::HeightMap)).await
            {
                Self::load_image(
                    heightmap,
                    &mut self.textures.heightmap_texture,
                    &mut self.images.heightmap_image_receiver,
                    &mut self.images.heightmap_image_handle,
                );
            }

            if let Ok(Some(terrain)) = map.send(GetMapImage::new(MapDisplayMode::Terrain)).await {
                Self::load_image(
                    terrain,
                    &mut self.textures.terrain_texture,
                    &mut self.images.terrain_image_receiver,
                    &mut self.images.terrain_image_handle,
                );
            }
            if let Ok(Some(provinces)) = map.send(GetMapImage::new(MapDisplayMode::Provinces)).await
            {
                Self::load_image(
                    provinces,
                    &mut self.textures.provinces_texture,
                    &mut self.images.provinces_image_receiver,
                    &mut self.images.provinces_image_handle,
                );
            }

            if let Ok(Some(rivers)) = map.send(GetMapImage::new(MapDisplayMode::Rivers)).await {
                Self::load_image(
                    rivers,
                    &mut self.textures.rivers_texture,
                    &mut self.images.rivers_image_receiver,
                    &mut self.images.rivers_image_handle,
                );
            }
        }
    }

    #[allow(clippy::as_conversions)]
    fn load_map_image(image: RgbImage) -> ColorImage {
        let size = [image.width() as usize, image.height() as usize];
        let image_buffer = DynamicImage::ImageRgb8(image).into_rgba8();
        let pixels = image_buffer.as_flat_samples();
        ColorImage::from_rgba_unmultiplied(size, pixels.as_slice())
    }

    fn load_image(
        image: RgbImage,
        texture: &mut Option<TextureHandle>,
        receiver: &mut Option<Receiver<ColorImage>>,
        handle: &mut Option<JoinHandle<()>>,
    ) {
        if texture.is_none() && handle.is_none() {
            let (tx, rx) = channel(1);
            *receiver = Some(rx);
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

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::as_conversions)]
    fn render_map(
        ui: &mut Ui,
        image: &mut Option<ColorImage>,
        texture: &mut Option<TextureHandle>,
        viewport: &mut Option<Rect>,
        zoom_level: &mut Option<f32>,
        selected_point: &mut Option<Pos2>,
        selected_province: &mut Option<Definition>,
    ) {
        if let Some(i) = image.take() {
            *texture = Some(ui.ctx().load_texture("map", i));
        }
        if let Some(tex) = &texture {
            let tex_size = tex.size_vec2();
            let size = ui.ctx().available_rect().size() * 0.8;

            let x_scale = size.x / tex_size.x;
            let y_scale = size.y / tex_size.y;
            let min_scale = x_scale.min(y_scale);
            let mut viewport_rect = viewport.map_or(
                Rect::from([Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)]),
                |r| r,
            );
            clamp_viewport(&mut viewport_rect);
            let image_button = ImageButton::new(tex, tex_size * min_scale)
                .uv(viewport_rect)
                .sense(Sense::click_and_drag());

            let map = ui.add(image_button);
            let map_rect = map.rect;
            let mouse_pos = ui.ctx().pointer_latest_pos();
            if let Some(pos) = mouse_pos {
                if map_rect.contains(pos) {
                    let scroll = Self::handle_scroll(ui, zoom_level);
                    Self::handle_zoom(viewport, zoom_level, viewport_rect, scroll);
                    Self::handle_drag(viewport, zoom_level, viewport_rect, &map);
                    let tex_uv = Self::project_to_texture(&viewport_rect, tex_size, pos, &map_rect);
                    ui.label(format!(
                        "Map Coordinate: ({:?}, {:?})",
                        tex_uv.x as i32, tex_uv.y as i32
                    ));
                    if map.clicked() {
                        selected_province.take();
                        *selected_point = Some(tex_uv);
                    }
                }
            }
        }
    }

    fn handle_drag(
        viewport: &mut Option<Rect>,
        zoom_level: &mut Option<f32>,
        mut viewport_rect: Rect,
        map: &Response,
    ) {
        let map_rect = map.rect;
        let mut map_drag = map.drag_delta();
        map_drag.x = map_drag.x / map_rect.width() * zoom_level.map_or(1.0, |z| 1.0 - z);
        map_drag.y = map_drag.y / map_rect.height() * zoom_level.map_or(1.0, |z| 1.0 - z);
        if map_drag.x != 0.0 || map_drag.y != 0.0 {
            let new_min =
                (viewport_rect.min - map_drag).clamp(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0));

            let new_max =
                (viewport_rect.max - map_drag).clamp(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0));

            let new_rect = Rect::from_min_max(new_min, new_max);

            if (new_rect.width() - viewport_rect.width()).abs() < f32::EPSILON
                && (new_rect.height() - viewport_rect.height()).abs() < f32::EPSILON
            {
                viewport_rect = Rect::from_min_max(new_min, new_max);
                *viewport = Some(viewport_rect);
            }
        }
    }

    fn handle_zoom(
        viewport: &mut Option<Rect>,
        zoom_level: &mut Option<f32>,
        mut viewport_rect: Rect,
        scroll: f32,
    ) {
        let mut zoomed_viewport = Rect::from_min_max(
            Pos2::new(
                zoom_level.map_or(0.0, |z| z / 2.0),
                zoom_level.map_or(0.0, |z| z / 2.0),
            ),
            Pos2::new(
                zoom_level.map_or(1.0, |z| 1.0 - z / 2.0),
                zoom_level.map_or(1.0, |z| 1.0 - z / 2.0),
            ),
        );
        let zoomed_viewport_center =
            zoomed_viewport.min + (zoomed_viewport.max - zoomed_viewport.min) / 2.0;

        let viewport_center = viewport_rect.min + (viewport_rect.max - viewport_rect.min) / 2.0;
        let translate = viewport_center - zoomed_viewport_center;

        if translate.length() > 0.00001 {
            zoomed_viewport.max =
                (zoomed_viewport.max + translate).clamp(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0));
            zoomed_viewport.min =
                (zoomed_viewport.min + translate).clamp(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0));
        }
        if scroll != 0.0 {
            viewport_rect = zoomed_viewport;
            *viewport = Some(viewport_rect);
        }
    }

    fn handle_scroll(ui: &mut Ui, zoom_level: &mut Option<f32>) -> f32 {
        let scroll = ui.input().scroll_delta.y;
        if scroll > 0.0 {
            *zoom_level = zoom_level.map_or(Some(0.01), |z| {
                if z < 0.7 {
                    Some(truncate_to_decimal_places((z + 0.01).min(0.99), 4))
                } else {
                    Some(truncate_to_decimal_places((z + 0.005).min(0.99), 4))
                }
            });
        }
        if scroll < 0.0 {
            *zoom_level = zoom_level.map_or(Some(0.01), |z| {
                if z < 0.7 {
                    Some(truncate_to_decimal_places((z - 0.01).max(0.0), 4))
                } else {
                    Some(truncate_to_decimal_places((z - 0.005).max(0.0), 4))
                }
            });
        }
        scroll
    }

    /// Projects a position from the UI space to the texture space.
    #[allow(clippy::similar_names)]
    fn project_to_texture(viewport: &Rect, tex_size: Vec2, pos: Pos2, map_rect: &Rect) -> Pos2 {
        // Get relative position of the map_rect
        let map_rect_u = pos.x - map_rect.min.x;
        let map_rect_v = pos.y - map_rect.min.y;

        // Viewports are clamped to the range [0, 1], so get the size of the viewport in pixels.
        let viewport_u_size = viewport.width() * tex_size.x;
        let viewport_v_size = viewport.height() * tex_size.y;

        // Get the relative scale of the viewport space and the ui space
        let viewport_map_u_scale = viewport_u_size / map_rect.width();
        let viewport_map_v_scale = viewport_v_size / map_rect.height();

        let viewport_u = viewport_map_u_scale * map_rect_u;
        let viewport_v = viewport_map_v_scale * map_rect_v;

        // Project viewport uv to texture uv
        let tex_u = viewport.min.x.mul_add(tex_size.x, viewport_u).round();
        let tex_v = viewport.min.y.mul_add(tex_size.y, viewport_v).round();
        Pos2::new(tex_u, tex_v)
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

    fn render_map_panel(&mut self, ctx: &Context, ui: &mut Ui) {
        match self.map_display_mode {
            MapDisplayMode::HeightMap => {
                Self::render_map(
                    ui,
                    &mut self.images.heightmap_image,
                    &mut self.textures.heightmap_texture,
                    &mut self.viewport,
                    &mut self.zoom_level,
                    &mut self.selected_point,
                    &mut self.selected_province,
                );
            }
            MapDisplayMode::Terrain => {
                Self::render_map(
                    ui,
                    &mut self.images.terrain_image,
                    &mut self.textures.terrain_texture,
                    &mut self.viewport,
                    &mut self.zoom_level,
                    &mut self.selected_point,
                    &mut self.selected_province,
                );
            }
            MapDisplayMode::Provinces => {
                Self::render_map(
                    ui,
                    &mut self.images.provinces_image,
                    &mut self.textures.provinces_texture,
                    &mut self.viewport,
                    &mut self.zoom_level,
                    &mut self.selected_point,
                    &mut self.selected_province,
                );
            }
            MapDisplayMode::Rivers => {
                Self::render_map(
                    ui,
                    &mut self.images.rivers_image,
                    &mut self.textures.rivers_texture,
                    &mut self.viewport,
                    &mut self.zoom_level,
                    &mut self.selected_point,
                    &mut self.selected_province,
                );
            }
            m => error!("Unknown Mode: {m}"),
        }
    }

    fn render_panels(&mut self, ctx: Context) -> Result<(), MapError> {
        if let Some(ui_renderer) = &self.ui_renderer {
            if let Some(rt) = &self.runtime {
                debug!("Render Loop start");
                let render_top_menu_bar = RenderTopMenuBar::new(ctx.clone());
                debug!("Block on TopMenubar");
                rt.block_on(ui_renderer.top_menu_renderer.send(render_top_menu_bar))??;
                let render_control_panel = RenderControlPanel::new(ctx.clone());
                debug!("Block on ControlPanel");
                rt.block_on(
                    ui_renderer
                        .control_panel_renderer
                        .send(render_control_panel),
                )??;
                let render_right_panel = RenderRightPanel::new(ctx.clone());
                debug!("Block on RightPanel");
                rt.block_on(ui_renderer.right_panel_renderer.send(render_right_panel))??;
                let render_central_panel = RenderCentralPanel::new(ctx.clone());
                rt.block_on(
                    ui_renderer
                        .central_panel_renderer
                        .send(render_central_panel),
                )??;
                debug!("Render Loop End");
            }
        }

        Ok(())
    }
}

impl App for WorldGenApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.initialize_renderer()
            .expect("Failed to initialize renderer");
        // Self::update_item(&mut self.map_receiver, &mut self.map, &mut self.map_handle);
        // Self::update_item(
        //     &mut self.root_path_receiver,
        //     &mut self.root_path,
        //     &mut self.root_path_handle,
        // );
        // Self::update_images(&mut self.images);

        let context = ctx.clone();
        let render_result = self.render_panels(context);
        if let Err(e) = render_result {
            error!("{:?}", e);
        }
        ctx.request_repaint();
    }

    fn on_exit(&mut self, _gl: &eframe::glow::Context) {
        debug!("on_exit");
        if let Some(s) = &self.system {
            s.stop();
        }
    }
}

fn clamp_viewport(mut viewport: &mut Rect) {
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
}

/// Truncates a floating point number to the specified number of decimal places.
#[must_use]
#[inline]
pub fn truncate_to_decimal_places(num: f32, places: i32) -> f32 {
    let ten = 10.0_f32.powi(places);
    // Need to check here because floats will become infinite if they are too large.  We are safe
    // to return `num` in this case because f64s cannot represent fractional values beyond 2^53.
    if num > f32::MAX / ten || num < f32::MIN / ten {
        return num;
    }
    (num * ten).floor() / ten
}

fn main() {
    env_logger::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(800.0, 600.0)),
        ..Default::default()
    };

    let app = WorldGenApp::default();

    eframe::run_native(
        "Hearts of Iron IV Map Editor",
        options,
        Box::new(|_cc| Box::new(app)),
    );
}
