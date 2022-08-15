use actix::{Actor, Context, Handler, Message, MessageResult};
use world_gen::MapDisplayMode;

/// A request to get the map display mode
#[derive(Message)]
#[rtype(result = "MapDisplayMode")]
#[non_exhaustive]
pub struct GetMapMode;

/// A request to set the map display mode
#[derive(Message)]
#[rtype(result = "()")]
#[non_exhaustive]
pub struct SetMapMode(pub MapDisplayMode);

impl SetMapMode {
    pub const fn new(mode: MapDisplayMode) -> Self {
        Self(mode)
    }
}

#[derive(Default, Debug)]
pub struct MapMode {
    mode: MapDisplayMode,
}

impl Actor for MapMode {
    type Context = Context<Self>;
}

impl Handler<GetMapMode> for MapMode {
    type Result = MessageResult<GetMapMode>;

    fn handle(&mut self, _msg: GetMapMode, _ctx: &mut Self::Context) -> Self::Result {
        MessageResult(self.mode)
    }
}

impl Handler<SetMapMode> for MapMode {
    type Result = ();

    fn handle(&mut self, msg: SetMapMode, _ctx: &mut Self::Context) -> Self::Result {
        self.mode = msg.0;
    }
}
