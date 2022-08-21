use actix::{Actor, AsyncContext, Context as ActixContext, Handler, Message};
use egui::{ColorImage, Context, TextureFilter, TextureHandle};
use image::{DynamicImage, RgbImage};
use tokio::task::JoinHandle;
use world_gen::MapDisplayMode;

/// A request to load an image
#[derive(Message)]
#[rtype(result = "()")]
pub enum LoadImage {
    HeightMap { image: RgbImage, context: Context },
    Terrain { image: RgbImage, context: Context },
    Provinces { image: RgbImage, context: Context },
    Rivers { image: RgbImage, context: Context },
    StrategicRegions { image: RgbImage, context: Context },
    States { image: RgbImage, context: Context },
}

impl LoadImage {
    pub const fn from_display_mode(
        mode: MapDisplayMode,
        image: RgbImage,
        context: Context,
    ) -> Self {
        match mode {
            MapDisplayMode::HeightMap => Self::HeightMap { image, context },
            MapDisplayMode::Terrain => Self::Terrain { image, context },
            MapDisplayMode::Provinces => Self::Provinces { image, context },
            MapDisplayMode::Rivers => Self::Rivers { image, context },
            MapDisplayMode::StrategicRegions => Self::StrategicRegions { image, context },
            MapDisplayMode::States => Self::States { image, context },
        }
    }
}

/// A request to update a texture
#[derive(Message)]
#[rtype(result = "()")]
enum UpdateTexture {
    HeightMap(TextureHandle),
    Terrain(TextureHandle),
    Provinces(TextureHandle),
    Rivers(TextureHandle),
    StrategicRegions(TextureHandle),
    States(TextureHandle),
}

/// A request to get a texture
#[derive(Message)]
#[rtype(result = "Option<TextureHandle>")]
#[non_exhaustive]
pub enum GetTexture {
    HeightMap,
    Terrain,
    Provinces,
    Rivers,
    StrategicRegions,
    States,
}

impl From<MapDisplayMode> for GetTexture {
    fn from(m: MapDisplayMode) -> Self {
        match m {
            MapDisplayMode::HeightMap => Self::HeightMap,
            MapDisplayMode::Terrain => Self::Terrain,
            MapDisplayMode::Provinces => Self::Provinces,
            MapDisplayMode::Rivers => Self::Rivers,
            MapDisplayMode::StrategicRegions => Self::StrategicRegions,
            MapDisplayMode::States => Self::States,
        }
    }
}

#[derive(Default)]
pub struct MapTextures {
    heightmap_texture: Option<TextureHandle>,
    terrain_texture: Option<TextureHandle>,
    provinces_texture: Option<TextureHandle>,
    rivers_texture: Option<TextureHandle>,
    strategic_regions_texture: Option<TextureHandle>,
    states_texture: Option<TextureHandle>,
    heightmap_handle: Option<JoinHandle<()>>,
    terrain_handle: Option<JoinHandle<()>>,
    provinces_handle: Option<JoinHandle<()>>,
    rivers_handle: Option<JoinHandle<()>>,
    strategic_regions_handle: Option<JoinHandle<()>>,
    states_handle: Option<JoinHandle<()>>,
}

impl Actor for MapTextures {
    type Context = ActixContext<Self>;
}

impl Handler<LoadImage> for MapTextures {
    type Result = ();

    fn handle(&mut self, msg: LoadImage, ctx: &mut Self::Context) -> Self::Result {
        let self_addr = ctx.address();
        match msg {
            LoadImage::HeightMap { image, context } => {
                if self.heightmap_handle.is_some() {
                    return;
                }
                self.heightmap_handle = Some(tokio::task::spawn_blocking(move || {
                    let tex = load_texture(image, &context);
                    self_addr.do_send(UpdateTexture::HeightMap(tex));
                }));
            }
            LoadImage::Terrain { image, context } => {
                if self.terrain_handle.is_some() {
                    return;
                }
                self.terrain_handle = Some(tokio::task::spawn_blocking(move || {
                    let tex = load_texture(image, &context);
                    self_addr.do_send(UpdateTexture::Terrain(tex));
                }));
            }
            LoadImage::Provinces { image, context } => {
                if self.provinces_handle.is_some() {
                    return;
                }
                self.provinces_handle = Some(tokio::task::spawn_blocking(move || {
                    let tex = load_texture(image, &context);
                    self_addr.do_send(UpdateTexture::Provinces(tex));
                }));
            }
            LoadImage::Rivers { image, context } => {
                if self.rivers_handle.is_some() {
                    return;
                }
                self.rivers_handle = Some(tokio::task::spawn_blocking(move || {
                    let tex = load_texture(image, &context);
                    self_addr.do_send(UpdateTexture::Rivers(tex));
                }));
            }
            LoadImage::StrategicRegions { image, context } => {
                if self.strategic_regions_handle.is_some() {
                    return;
                }
                self.strategic_regions_handle = Some(tokio::task::spawn_blocking(move || {
                    let tex = load_texture(image, &context);
                    self_addr.do_send(UpdateTexture::StrategicRegions(tex));
                }));
            }
            LoadImage::States { image, context } => {
                if self.states_handle.is_some() {
                    return;
                }
                self.states_handle = Some(tokio::task::spawn_blocking(move || {
                    let tex = load_texture(image, &context);
                    self_addr.do_send(UpdateTexture::States(tex));
                }));
            }
        };
    }
}

fn load_texture(rgb_image: RgbImage, context: &Context) -> TextureHandle {
    let size = [rgb_image.width() as usize, rgb_image.height() as usize];
    let image_buffer = DynamicImage::ImageRgb8(rgb_image).into_rgba8();
    let pixels = image_buffer.as_flat_samples();
    let color_image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
    context.load_texture("map", color_image, TextureFilter::Nearest)
}

impl Handler<GetTexture> for MapTextures {
    type Result = Option<TextureHandle>;

    fn handle(&mut self, msg: GetTexture, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            GetTexture::HeightMap => self.heightmap_texture.clone(),
            GetTexture::Terrain => self.terrain_texture.clone(),
            GetTexture::Provinces => self.provinces_texture.clone(),
            GetTexture::Rivers => self.rivers_texture.clone(),
            GetTexture::StrategicRegions => self.strategic_regions_texture.clone(),
            GetTexture::States => self.states_texture.clone(),
        }
    }
}

impl Handler<UpdateTexture> for MapTextures {
    type Result = ();

    fn handle(&mut self, msg: UpdateTexture, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            UpdateTexture::HeightMap(t) => {
                self.heightmap_texture = Some(t);
                self.heightmap_handle.take();
            }
            UpdateTexture::Terrain(t) => {
                self.terrain_texture = Some(t);
                self.terrain_handle.take();
            }
            UpdateTexture::Provinces(t) => {
                self.provinces_texture = Some(t);
                self.provinces_handle.take();
            }
            UpdateTexture::Rivers(t) => {
                self.rivers_texture = Some(t);
                self.rivers_handle.take();
            }
            UpdateTexture::StrategicRegions(t) => {
                self.strategic_regions_texture = Some(t);
                self.strategic_regions_handle.take();
            }
            UpdateTexture::States(t) => {
                self.states_texture = Some(t);
                self.states_handle.take();
            }
        }
    }
}
