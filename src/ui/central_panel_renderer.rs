use crate::ui::map_loader::GetMap;
use crate::ui::map_mode::GetMapMode;
use crate::ui::map_textures::{GetTexture, LoadImage};
use crate::ui::selection::SetSelectedPoint;
use crate::ui::viewport::{GetViewportArea, GetZoomLevel, Scroll, SetViewportArea};
use crate::{MapError, MapLoader, MapMode, MapTextures, Selection, Viewport};
use actix::{Actor, Addr, AsyncContext, Context as ActixContext, Handler, Message, ResponseFuture};
use egui::{
    CentralPanel, Context, ImageButton, Pos2, Rect, Response, Sense, TextureHandle, Ui, Vec2,
};
use world_gen::map::{GetMapImage, Map};
use world_gen::MapDisplayMode;

/// A request to render the right panel
#[derive(Message)]
#[rtype(result = "Result<(), MapError>")]
#[non_exhaustive]
pub struct RenderCentralPanel {
    pub context: Context,
}

impl RenderCentralPanel {
    #[inline]
    pub const fn new(context: Context) -> Self {
        Self { context }
    }
}

/// A request to set the address of the map
#[derive(Message)]
#[rtype(result = "()")]
#[non_exhaustive]
pub struct SetMap(pub Addr<Map>);

#[derive(Debug)]
pub struct CentralPanelRenderer {
    map_loader: Addr<MapLoader>,
    map_mode: Addr<MapMode>,
    map_textures: Addr<MapTextures>,
    selection: Addr<Selection>,
    map: Option<Addr<Map>>,
    viewport: Addr<Viewport>,
}

impl CentralPanelRenderer {
    #[inline]
    pub const fn new(
        map_loader: Addr<MapLoader>,
        map_mode: Addr<MapMode>,
        map_textures: Addr<MapTextures>,
        selection: Addr<Selection>,
        viewport: Addr<Viewport>,
    ) -> Self {
        Self {
            map_loader,
            map_mode,
            map_textures,
            selection,
            map: None,
            viewport,
        }
    }
}

impl Actor for CentralPanelRenderer {
    type Context = ActixContext<Self>;
}

impl Handler<RenderCentralPanel> for CentralPanelRenderer {
    type Result = ResponseFuture<Result<(), MapError>>;

    fn handle(&mut self, msg: RenderCentralPanel, ctx: &mut Self::Context) -> Self::Result {
        let context = msg.context;
        let map_loader_addr = self.map_loader.clone();
        let map_addr = self.map.clone();
        let map_mode_addr = self.map_mode.clone();
        let map_textures_addr = self.map_textures.clone();
        let selection_addr = self.selection.clone();
        let viewport_addr = self.viewport.clone();
        let self_addr = ctx.address();
        Box::pin(async move {
            let map_mode: MapDisplayMode = map_mode_addr.send(GetMapMode).await?;
            let texture: Option<TextureHandle> =
                map_textures_addr.send(GetTexture::from(map_mode)).await?;
            if map_addr.is_none() {
                let addr = map_loader_addr.send(GetMap).await?;
                if let Some(m) = addr {
                    self_addr.do_send(SetMap(m));
                }
            }
            if let (Some(map), None) = (map_addr.clone(), texture.clone()) {
                if let Some(image) = map.send(GetMapImage::from(map_mode)).await? {
                    map_textures_addr
                        .send(LoadImage::from_display_mode(
                            map_mode,
                            image,
                            context.clone(),
                        ))
                        .await?;
                }
            }
            let viewport_rect: Rect = viewport_addr.send(GetViewportArea).await?.map_or(
                Rect::from([Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)]),
                |r| r,
            );
            let zoom_level = viewport_addr.send(GetZoomLevel).await?;

            let mut selected_point = None;
            CentralPanel::default().show(&context, |ui| {
                if let Some(tex) = &texture {
                    let tex_size = tex.size_vec2();
                    let size = ui.ctx().available_rect().size() * 0.8;
                    let x_scale = size.x / tex_size.x;
                    let y_scale = size.y / tex_size.y;
                    let min_scale = x_scale.min(y_scale);
                    let image_button = ImageButton::new(tex, tex_size * min_scale)
                        .uv(viewport_rect)
                        .sense(Sense::click_and_drag());
                    let map = ui.add(image_button);
                    let map_rect = map.rect;
                    let mouse_pos = ui.ctx().pointer_latest_pos();
                    if let Some(pos) = mouse_pos {
                        if map_rect.contains(pos) {
                            let scroll = handle_scroll(ui, &viewport_addr);
                            handle_zoom(&viewport_addr, zoom_level, viewport_rect, scroll);
                            handle_drag(&viewport_addr, zoom_level, viewport_rect, &map);
                            let tex_uv =
                                project_to_texture(&viewport_rect, tex_size, pos, &map_rect);
                            ui.label(format!(
                                "Map Coordinate: ({:?}, {:?})",
                                tex_uv.x as i32, tex_uv.y as i32
                            ));
                            if map.clicked() {
                                selected_point = Some(tex_uv);
                            }
                        }
                    }
                } else if map_addr.is_some() {
                    ui.label("Loading...");
                }
            });
            if let Some(point) = selected_point {
                selection_addr.send(SetSelectedPoint::new(point)).await?;
            }
            Ok(())
        })
    }
}

impl Handler<SetMap> for CentralPanelRenderer {
    type Result = ();

    fn handle(&mut self, msg: SetMap, _ctx: &mut Self::Context) -> Self::Result {
        self.map = Some(msg.0);
    }
}

fn handle_scroll(ui: &mut Ui, viewport: &Addr<Viewport>) -> f32 {
    let scroll = ui.input().scroll_delta.y;
    viewport.do_send(Scroll(scroll));
    scroll
}

fn handle_zoom(
    viewport: &Addr<Viewport>,
    zoom_level: Option<f32>,
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
        viewport.do_send(SetViewportArea(viewport_rect));
    }
}

fn handle_drag(
    viewport: &Addr<Viewport>,
    zoom_level: Option<f32>,
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
            viewport.do_send(SetViewportArea(viewport_rect));
        }
    }
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
