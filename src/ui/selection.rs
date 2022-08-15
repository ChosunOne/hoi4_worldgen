use actix::{Actor, Context, Handler, Message};
use egui::Pos2;
use world_gen::components::prelude::Definition;

/// A request to get the selected point
#[derive(Message)]
#[rtype(result = "Option<Pos2>")]
#[non_exhaustive]
pub struct GetSelectedPoint;

/// A request to set the selected point
#[derive(Message)]
#[rtype(result = "()")]
#[non_exhaustive]
pub struct SetSelectedPoint(pub Pos2);

/// A request to get the selected province definition
#[derive(Message)]
#[rtype(result = "Option<Definition>")]
#[non_exhaustive]
pub struct GetSelectedProvince;

/// A request to set the selected province definition
#[derive(Message)]
#[rtype(result = "()")]
#[non_exhaustive]
pub struct SetSelectedProvince(pub Definition);

impl SetSelectedProvince {
    #[inline]
    pub const fn new(definition: Definition) -> Self {
        Self(definition)
    }
}

impl SetSelectedPoint {
    pub const fn new(point: Pos2) -> Self {
        Self(point)
    }
}

#[derive(Default, Debug)]
pub struct Selection {
    selected_point: Option<Pos2>,
    selected_province: Option<Definition>,
}
impl Actor for Selection {
    type Context = Context<Self>;
}

impl Handler<GetSelectedPoint> for Selection {
    type Result = Option<Pos2>;

    fn handle(&mut self, _msg: GetSelectedPoint, _ctx: &mut Self::Context) -> Self::Result {
        self.selected_point
    }
}

impl Handler<SetSelectedPoint> for Selection {
    type Result = ();

    fn handle(&mut self, msg: SetSelectedPoint, _ctx: &mut Self::Context) -> Self::Result {
        self.selected_point = Some(msg.0);
        self.selected_province.take();
    }
}

impl Handler<GetSelectedProvince> for Selection {
    type Result = Option<Definition>;

    fn handle(&mut self, _msg: GetSelectedProvince, _ctx: &mut Self::Context) -> Self::Result {
        self.selected_province.clone()
    }
}

impl Handler<SetSelectedProvince> for Selection {
    type Result = ();

    fn handle(&mut self, msg: SetSelectedProvince, _ctx: &mut Self::Context) -> Self::Result {
        self.selected_province = Some(msg.0);
    }
}
