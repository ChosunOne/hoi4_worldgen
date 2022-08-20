use actix::{Actor, Context, Handler, Message};
use egui::Pos2;
use world_gen::components::prelude::{Definition, StrategicRegion};
use world_gen::components::state::State;

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

/// A request to get the selected state
#[derive(Message)]
#[rtype(result = "Option<State>")]
#[non_exhaustive]
pub struct GetSelectedState;

/// A request to set the selected state
#[derive(Message)]
#[rtype(result = "()")]
#[non_exhaustive]
pub struct SetSelectedState(pub State);

/// A request to get the selected strategic region
#[derive(Message)]
#[rtype(result = "Option<StrategicRegion>")]
#[non_exhaustive]
pub struct GetSelectedStrategicRegion;

/// A request to set the selected strategic region
#[derive(Message)]
#[rtype(result = "()")]
#[non_exhaustive]
pub struct SetSelectedStrategicRegion(pub StrategicRegion);

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

impl SetSelectedState {
    pub const fn new(state: State) -> Self {
        Self(state)
    }
}

impl SetSelectedStrategicRegion {
    pub const fn new(region: StrategicRegion) -> Self {
        Self(region)
    }
}

#[derive(Default, Debug)]
pub struct Selection {
    selected_point: Option<Pos2>,
    selected_province: Option<Definition>,
    selected_state: Option<State>,
    selected_strategic_region: Option<StrategicRegion>,
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
        self.selected_state.take();
        self.selected_strategic_region.take();
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

impl Handler<GetSelectedState> for Selection {
    type Result = Option<State>;

    fn handle(&mut self, _msg: GetSelectedState, _ctx: &mut Self::Context) -> Self::Result {
        self.selected_state.clone()
    }
}

impl Handler<SetSelectedState> for Selection {
    type Result = ();

    fn handle(&mut self, msg: SetSelectedState, _ctx: &mut Self::Context) -> Self::Result {
        self.selected_state = Some(msg.0);
    }
}

impl Handler<GetSelectedStrategicRegion> for Selection {
    type Result = Option<StrategicRegion>;

    fn handle(
        &mut self,
        _msg: GetSelectedStrategicRegion,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.selected_strategic_region.clone()
    }
}

impl Handler<SetSelectedStrategicRegion> for Selection {
    type Result = ();

    fn handle(
        &mut self,
        msg: SetSelectedStrategicRegion,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.selected_strategic_region = Some(msg.0);
    }
}
