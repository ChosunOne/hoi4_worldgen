use crate::ui::map_loader::{GetMap, IsMapLoading, LoadMap, MapLoader};
use crate::ui::map_mode::SetMapMode;
use crate::ui::map_textures::{GetTexture, LoadImage};
use crate::ui::root_path::GetRootPath;
use crate::{MapError, MapMode, MapTextures, RootPath};
use actix::Addr;
use eframe::epaint::TextureHandle;
use egui::{Context, TopBottomPanel};
use image::RgbImage;
use indicatif::InMemoryTerm;
use log::{debug, error, trace};
use std::path::PathBuf;
use tokio::{join, try_join};
use world_gen::map::{GetMapImage, Map};
use world_gen::MapDisplayMode;

pub struct ControlPanelRenderer {
    root_path: Addr<RootPath>,
    map_loader: Addr<MapLoader>,
    map_mode: Addr<MapMode>,
    map_textures: Addr<MapTextures>,
    terminal: InMemoryTerm,
}

struct TextureHandles {
    heightmap: Option<TextureHandle>,
    terrain: Option<TextureHandle>,
    rivers: Option<TextureHandle>,
    provinces: Option<TextureHandle>,
    states: Option<TextureHandle>,
    strategic_regions: Option<TextureHandle>,
}

impl TextureHandles {
    pub async fn new(map_textures: &Addr<MapTextures>) -> Result<Self, MapError> {
        // The type for these are Option<TextureHandle>
        let (
            heightmap_texture,
            terrain_texture,
            rivers_texture,
            provinces_texture,
            states_texture,
            strategic_regions_texture,
        ) = try_join!(
            map_textures.send(GetTexture::HeightMap),
            map_textures.send(GetTexture::Terrain),
            map_textures.send(GetTexture::Rivers),
            map_textures.send(GetTexture::Provinces),
            map_textures.send(GetTexture::States),
            map_textures.send(GetTexture::StrategicRegions)
        )?;

        Ok(Self {
            heightmap: heightmap_texture,
            terrain: terrain_texture,
            rivers: rivers_texture,
            provinces: provinces_texture,
            states: states_texture,
            strategic_regions: strategic_regions_texture,
        })
    }
}

impl ControlPanelRenderer {
    #[inline]
    pub const fn new(
        root_path: Addr<RootPath>,
        map_loader: Addr<MapLoader>,
        map_mode: Addr<MapMode>,
        map_textures: Addr<MapTextures>,
        terminal: InMemoryTerm,
    ) -> Self {
        Self {
            root_path,
            map_loader,
            map_mode,
            map_textures,
            terminal,
        }
    }

    #[allow(clippy::integer_arithmetic)]
    #[allow(clippy::too_many_lines)]
    pub async fn render_control_panel(&self, ctx: &Context) -> Result<(), MapError> {
        let root_path: Option<PathBuf> = self.root_path.send(GetRootPath).await?;
        let map: Option<Addr<Map>> = self.map_loader.send(GetMap).await?;

        let texture_handles = TextureHandles::new(&self.map_textures).await?;

        let is_map_loading = self.map_loader.send(IsMapLoading).await?;
        self.load_textures(ctx, &map, &texture_handles, is_map_loading)
            .await?;
        TopBottomPanel::top("control_panel").show(ctx, |ui| {
            if let Some(pathbuf) = root_path {
                ui.horizontal(|ui| {
                    ui.label("Root Directory: ");
                    ui.label(pathbuf.display().to_string());
                    if map.is_none() && ui.button("Load Map").clicked() {
                        if let Err(e) = self
                            .map_loader
                            .try_send(LoadMap::new(pathbuf, self.terminal.clone()))
                        {
                            error!("{e}");
                        }
                    }
                });
                if is_map_loading {
                    ui.spinner();
                }
            } else {
                ui.heading("Please select a root folder");
            }
            if map.is_some() {
                ui.horizontal(|ui| {
                    if texture_handles.heightmap.is_some() {
                        if ui.button("Height Map").clicked() {
                            self.map_mode
                                .do_send(SetMapMode::new(MapDisplayMode::HeightMap));
                        }
                    } else {
                        ui.spinner();
                    }

                    if texture_handles.terrain.is_some() {
                        if ui.button("Terrain").clicked() {
                            self.map_mode
                                .do_send(SetMapMode::new(MapDisplayMode::Terrain));
                        }
                    } else {
                        ui.spinner();
                    }

                    if texture_handles.rivers.is_some() {
                        if ui.button("Rivers").clicked() {
                            self.map_mode
                                .do_send(SetMapMode::new(MapDisplayMode::Rivers));
                        }
                    } else {
                        ui.spinner();
                    }

                    if texture_handles.provinces.is_some() {
                        if ui.button("Provinces").clicked() {
                            self.map_mode
                                .do_send(SetMapMode::new(MapDisplayMode::Provinces));
                        }
                    } else {
                        ui.spinner();
                    }

                    if texture_handles.states.is_some() {
                        if ui.button("States").clicked() {
                            self.map_mode
                                .do_send(SetMapMode::new(MapDisplayMode::States));
                        }
                    } else {
                        ui.spinner();
                    }

                    if texture_handles.strategic_regions.is_some() {
                        if ui.button("Strategic Regions").clicked() {
                            self.map_mode
                                .do_send(SetMapMode::new(MapDisplayMode::StrategicRegions));
                        }
                    } else {
                        ui.spinner();
                    }
                });
            }
        });
        Ok(())
    }

    async fn load_textures(
        &self,
        ctx: &Context,
        map: &Option<Addr<Map>>,
        texture_handles: &TextureHandles,
        is_map_loading: bool,
    ) -> Result<(), MapError> {
        if let Some(m) = &map {
            if !is_map_loading {
                if texture_handles.heightmap.is_none() {
                    if let Some(image) = m.send(GetMapImage::HeightMap).await? {
                        self.map_textures.do_send(LoadImage::HeightMap {
                            image,
                            context: ctx.clone(),
                        });
                    }
                }

                if texture_handles.terrain.is_none() {
                    if let Some(image) = m.send(GetMapImage::Terrain).await? {
                        self.map_textures.do_send(LoadImage::Terrain {
                            image,
                            context: ctx.clone(),
                        });
                    }
                }

                if texture_handles.rivers.is_none() {
                    if let Some(image) = m.send(GetMapImage::Rivers).await? {
                        self.map_textures
                            .send(LoadImage::Rivers {
                                image,
                                context: ctx.clone(),
                            })
                            .await?;
                    }
                }

                if texture_handles.provinces.is_none() {
                    if let Some(image) = m.send(GetMapImage::Provinces).await? {
                        self.map_textures
                            .send(LoadImage::Provinces {
                                image,
                                context: ctx.clone(),
                            })
                            .await?;
                    }
                }

                if texture_handles.states.is_none() {
                    if let Some(image) = m.send(GetMapImage::States).await? {
                        self.map_textures
                            .send(LoadImage::States {
                                image,
                                context: ctx.clone(),
                            })
                            .await?;
                    }
                }

                if texture_handles.strategic_regions.is_none() {
                    if let Some(image) = m.send(GetMapImage::StrategicRegions).await? {
                        self.map_textures
                            .send(LoadImage::StrategicRegions {
                                image,
                                context: ctx.clone(),
                            })
                            .await?;
                    }
                }
            }
        }

        Ok(())
    }
}
