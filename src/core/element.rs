use crate::core::common::*;

use crate::space::Area;

mod context;
pub(crate) use self::context::Children;
pub use self::context::{Context, SubContext};

mod id;
pub use self::id::Id;

mod socket;
pub use self::socket::{Socket, SocketName};

mod layout;
pub use self::layout::{Layout, LayoutObj};

pub struct Builder<T: Element> {
    pub id: Id,
    pub socket: SocketName,
    pub e: T,
}

// An 'Element' is something run in the the context of a socket
// This is the starting point for any UI tree
pub trait Element {
    fn run(&self, ctx: Context, id: Id) -> LayoutObj;
}

impl Element for () {
    fn run<'window>(&self, _ctx: Context<'window>, _id: Id) -> LayoutObj {
        LayoutObj::new(Area::zero(), ()).upcast()
    }
}

pub trait ElementExt: Element {
    fn begin<'a, 'ctx, 'window>(
        self,
        sub_ctx: &'a mut SubContext<'ctx, 'window>,
        socket: SocketName,
        id: Id,
    ) -> &'a mut SubContext<'ctx, 'window>;
}

impl<T: Element + 'static> ElementExt for T {
    fn begin<'a, 'ctx, 'window>(
        self,
        sub_ctx: &'a mut SubContext<'ctx, 'window>,
        socket: SocketName,
        id: Id,
    ) -> &'a mut SubContext<'ctx, 'window> {
        sub_ctx.begin(socket, id, self)
    }
}
