use actix::{Actor, AsyncContext, Context, Handler, Message};
use log::{debug, trace};
use std::path::PathBuf;
use tokio::task::JoinHandle;

/// A request to set the root path
#[derive(Message)]
#[rtype(result = "()")]
#[non_exhaustive]
pub struct SetRootPath;

/// A request to get the root path
#[derive(Message)]
#[rtype(result = "Option<PathBuf>")]
#[non_exhaustive]
pub struct GetRootPath;

/// A request to get the root path
#[derive(Message)]
#[rtype(result = "()")]
#[non_exhaustive]
pub struct UpdateRootPath(Option<PathBuf>);

impl UpdateRootPath {
    pub const fn new(pathbuf: Option<PathBuf>) -> Self {
        Self(pathbuf)
    }
}

#[derive(Default)]
pub struct RootPath {
    root_path: Option<PathBuf>,
    root_path_handle: Option<JoinHandle<()>>,
}

impl Actor for RootPath {
    type Context = Context<Self>;
}

impl Handler<SetRootPath> for RootPath {
    type Result = ();

    fn handle(&mut self, _msg: SetRootPath, ctx: &mut Self::Context) -> Self::Result {
        trace!("SetRootPath");
        if self.root_path_handle.is_some() {
            return;
        }
        let self_addr = ctx.address();
        let handle = tokio::task::spawn_blocking(move || {
            let path = rfd::FileDialog::new().pick_folder();
            self_addr.do_send(UpdateRootPath::new(path));
        });
        self.root_path_handle = Some(handle);
    }
}

impl Handler<GetRootPath> for RootPath {
    type Result = Option<PathBuf>;

    fn handle(&mut self, _msg: GetRootPath, _ctx: &mut Self::Context) -> Self::Result {
        trace!("GetRootPath");
        self.root_path.clone()
    }
}

impl Handler<UpdateRootPath> for RootPath {
    type Result = ();

    fn handle(&mut self, msg: UpdateRootPath, _ctx: &mut Self::Context) -> Self::Result {
        trace!("UpdateRootPath");
        self.root_path = msg.0;
        self.root_path_handle.take();
    }
}
