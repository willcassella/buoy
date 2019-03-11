use crate::prelude::*;

pub trait Panel {
    fn open(
        &self,
        max_area: Area
    ) -> Area;

    fn close(
        self,
        ctx: &mut Context,
        socket: &mut dyn Socket,
        children: Vec<LayoutObj>
    );
}

pub fn panel<T: Panel>(
    panel: T,
    ctx: &mut Context,
    socket: &mut dyn Socket,
) {
    let mut children = Vec::new();

    let child_max_area = panel.open(ctx.max_area());
    while ctx.socket(SocketName::default(), &mut children, child_max_area) { }

    panel.close(ctx, socket, children);
}
