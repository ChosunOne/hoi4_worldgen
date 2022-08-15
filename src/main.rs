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
use crate::ui::map_textures::MapTextures;
use crate::ui::right_panel_renderer::{RenderRightPanel, RightPanelRenderer};
use crate::ui::root_path::RootPath;
use crate::ui::selection::Selection;
use crate::ui::top_menu_renderer::{RenderTopMenuBar, TopMenuRenderer};
use crate::ui::viewport::Viewport;
use crate::ui::{root_path::SetRootPath, UiRenderer};
use actix::{Actor, System};
use eframe::App;
use egui::{Context, Vec2};
use indicatif::InMemoryTerm;
use log::{debug, error, info, trace};
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use world_gen::MapError;

struct WorldGenApp {
    system: Option<System>,
    terminal: InMemoryTerm,
    ui_renderer: Option<UiRenderer>,
    runtime: Option<Runtime>,
    system_thread: Option<JoinHandle<Result<(), MapError>>>,
}

impl Default for WorldGenApp {
    fn default() -> Self {
        Self {
            terminal: InMemoryTerm::new(16, 240),
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
            trace!("Spawning system");
            let system = System::new();

            system.block_on(async {
                trace!("Starting root path");
                let root_path = RootPath::default().start();
                trace!("Starting top menu renderer");
                let top_menu_renderer = TopMenuRenderer::new(root_path.clone()).start();
                trace!("Starting map textures");
                let map_textures = MapTextures::default().start();
                trace!("Starting map loader");
                let map_loader = MapLoader::default().start();
                trace!("Starting map mode");
                let map_mode = MapMode::default().start();
                trace!("Starting control panel renderer");
                let control_panel_renderer = ControlPanelRenderer::new(
                    root_path,
                    map_loader.clone(),
                    map_mode.clone(),
                    terminal.clone(),
                )
                .start();
                trace!("Starting selection");
                let selection = Selection::default().start();
                trace!("Starting right panel renderer");
                let right_panel_renderer = RightPanelRenderer::new(
                    map_mode.clone(),
                    selection.clone(),
                    map_loader.clone(),
                    terminal,
                )
                .start();
                trace!("Starting viewport");
                let viewport = Viewport::default().start();
                trace!("Starting central panel renderer");
                let central_panel_renderer = CentralPanelRenderer::new(
                    map_loader,
                    map_mode.clone(),
                    map_textures,
                    selection,
                    viewport.clone(),
                )
                .start();

                let ui_renderer = UiRenderer::new(
                    top_menu_renderer,
                    control_panel_renderer,
                    right_panel_renderer,
                    central_panel_renderer,
                    map_mode,
                    viewport,
                );
                trace!("Sending Ui Renderer");
                tx.send(ui_renderer).unwrap();
            });

            system_tx.send(System::current()).unwrap();

            trace!("Running system");
            system.run()?;
            trace!("System stopped");
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

    fn render_panels(&mut self, ctx: &Context) -> Result<(), MapError> {
        if let Some(ui_renderer) = &self.ui_renderer {
            if let Some(rt) = &self.runtime {
                trace!("Render Loop start");
                let render_top_menu_bar = RenderTopMenuBar::new(ctx.clone());
                trace!("Block on TopMenubar");
                rt.block_on(ui_renderer.top_menu_renderer.send(render_top_menu_bar))??;
                let render_control_panel = RenderControlPanel::new(ctx.clone());
                trace!("Block on ControlPanel");
                rt.block_on(
                    ui_renderer
                        .control_panel_renderer
                        .send(render_control_panel),
                )??;
                let render_right_panel = RenderRightPanel::new(ctx.clone());
                trace!("Block on RightPanel");
                rt.block_on(ui_renderer.right_panel_renderer.send(render_right_panel))??;
                let render_central_panel = RenderCentralPanel::new(ctx.clone());
                rt.block_on(
                    ui_renderer
                        .central_panel_renderer
                        .send(render_central_panel),
                )??;
                trace!("Render Loop End");
            }
        }

        Ok(())
    }
}

impl App for WorldGenApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.initialize_renderer()
            .expect("Failed to initialize renderer");

        let render_result = self.render_panels(ctx);
        if let Err(e) = render_result {
            error!("{:?}", e);
        }
        ctx.request_repaint();
    }

    fn on_exit(&mut self, _gl: &eframe::glow::Context) {
        trace!("on_exit");
        if let Some(s) = &self.system {
            s.stop();
        }
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
