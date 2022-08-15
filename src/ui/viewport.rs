use crate::truncate_to_decimal_places;
use actix::{Actor, Context, Handler, Message};
use egui::Rect;
use std::mem::swap;

/// A request to get the zoom level
#[derive(Message)]
#[rtype(result = "Option<f32>")]
#[non_exhaustive]
pub struct GetZoomLevel;

/// A request to set the zoom level
#[derive(Message)]
#[rtype(result = "()")]
#[non_exhaustive]
pub struct SetZoomLevel(f32);

/// A request to set the zoom level
#[derive(Message)]
#[rtype(result = "()")]
pub struct Scroll(pub f32);

/// A request to get the viewport area
#[derive(Message)]
#[rtype(result = "Option<Rect>")]
#[non_exhaustive]
pub struct GetViewportArea;

/// A request to set the viewport area
#[derive(Message)]
#[rtype(result = "()")]
pub struct SetViewportArea(pub Rect);

#[derive(Default, Debug)]
pub struct Viewport {
    zoom_level: Option<f32>,
    viewport_area: Option<Rect>,
}

impl Actor for Viewport {
    type Context = Context<Self>;
}

impl Handler<GetZoomLevel> for Viewport {
    type Result = Option<f32>;

    fn handle(&mut self, _msg: GetZoomLevel, _ctx: &mut Self::Context) -> Self::Result {
        self.zoom_level
    }
}

impl Handler<SetZoomLevel> for Viewport {
    type Result = ();

    fn handle(&mut self, msg: SetZoomLevel, _ctx: &mut Self::Context) -> Self::Result {
        self.zoom_level = Some(msg.0);
    }
}

impl Handler<GetViewportArea> for Viewport {
    type Result = Option<Rect>;

    fn handle(&mut self, _msg: GetViewportArea, _ctx: &mut Self::Context) -> Self::Result {
        self.viewport_area
    }
}

impl Handler<SetViewportArea> for Viewport {
    type Result = ();

    fn handle(&mut self, msg: SetViewportArea, _ctx: &mut Self::Context) -> Self::Result {
        let mut rect = msg.0;
        clamp_viewport(&mut rect);
        self.viewport_area = Some(rect);
    }
}

impl Handler<Scroll> for Viewport {
    type Result = ();

    fn handle(&mut self, msg: Scroll, ctx: &mut Self::Context) -> Self::Result {
        let scroll = msg.0;
        if scroll > 0.0 {
            self.zoom_level = self.zoom_level.map_or(Some(0.01), |z| {
                if z < 0.7 {
                    Some(truncate_to_decimal_places((z + 0.01).min(0.99), 4))
                } else {
                    Some(truncate_to_decimal_places((z + 0.005).min(0.99), 4))
                }
            });
        }
        if scroll < 0.0 {
            self.zoom_level = self.zoom_level.map_or(Some(0.01), |z| {
                if z < 0.7 {
                    Some(truncate_to_decimal_places((z - 0.01).max(0.0), 4))
                } else {
                    Some(truncate_to_decimal_places((z - 0.005).max(0.0), 4))
                }
            });
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
