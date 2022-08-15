use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};
use indicatif::InMemoryTerm;
use log::{debug, error};
use std::path::PathBuf;
use tokio::task::JoinHandle;
use world_gen::map::Map;
use world_gen::MapError;

/// A request to load the map
#[derive(Message)]
#[rtype(result = "()")]
#[non_exhaustive]
pub struct LoadMap {
    root_path: PathBuf,
    terminal: InMemoryTerm,
}

impl LoadMap {
    pub const fn new(root_path: PathBuf, terminal: InMemoryTerm) -> Self {
        Self {
            root_path,
            terminal,
        }
    }
}

/// A request to get the map
#[derive(Message)]
#[rtype(result = "Option<Addr<Map>>")]
#[non_exhaustive]
pub struct GetMap;

/// A request to see if the map is loading
#[derive(Message)]
#[rtype(result = "bool")]
#[non_exhaustive]
pub struct IsMapLoading;

/// A request to update the map
#[derive(Message)]
#[rtype(result = "()")]
#[non_exhaustive]
pub struct UpdateMap(Result<Map, MapError>);

impl UpdateMap {
    pub const fn new(map: Result<Map, MapError>) -> Self {
        Self(map)
    }
}

/// A request to check if the map has been loaded
#[derive(Message)]
#[rtype(result = "bool")]
#[non_exhaustive]
pub struct IsMapLoaded;

#[derive(Debug, Default)]
pub struct MapLoader {
    map: Option<Addr<Map>>,
    map_handle: Option<JoinHandle<()>>,
}

impl Actor for MapLoader {
    type Context = Context<Self>;
}

impl Handler<GetMap> for MapLoader {
    type Result = Option<Addr<Map>>;

    fn handle(&mut self, _msg: GetMap, _ctx: &mut Self::Context) -> Self::Result {
        debug!("GetMap");
        self.map.as_ref().cloned()
    }
}

impl Handler<UpdateMap> for MapLoader {
    type Result = ();

    fn handle(&mut self, msg: UpdateMap, _ctx: &mut Self::Context) -> Self::Result {
        debug!("UpdateMap");
        match msg.0 {
            Ok(m) => {
                self.map = Some(m.start());
                self.map_handle.take();
            }
            Err(e) => error!("{e:?}"),
        }
    }
}

impl Handler<LoadMap> for MapLoader {
    type Result = ();

    fn handle(&mut self, msg: LoadMap, ctx: &mut Self::Context) -> Self::Result {
        debug!("LoadMap");
        if self.map_handle.is_some() {
            return;
        }
        let self_addr = ctx.address();
        let map_loading_handle = tokio::task::spawn_blocking(move || {
            let map = Map::new(&msg.root_path, &Some(msg.terminal));
            self_addr.do_send(UpdateMap::new(map));
        });
        self.map_handle = Some(map_loading_handle);
    }
}

impl Handler<IsMapLoaded> for MapLoader {
    type Result = bool;

    fn handle(&mut self, _msg: IsMapLoaded, _ctx: &mut Self::Context) -> Self::Result {
        debug!("IsMapLoaded: {}", self.map.is_some());
        self.map.is_some()
    }
}

impl Handler<IsMapLoading> for MapLoader {
    type Result = bool;

    fn handle(&mut self, _msg: IsMapLoading, _ctx: &mut Self::Context) -> Self::Result {
        debug!("IsMapLoading: {}", self.map_handle.is_some());
        self.map_handle.is_some()
    }
}
