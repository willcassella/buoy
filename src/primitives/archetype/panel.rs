use crate::prelude::*;

pub trait Panel {
    fn open(
        &self,
        max_area: Area
    ) -> Area;

    fn close<'window, 'ctx>(
        &self,
        ctx: Context<'window, 'ctx>,
        children: Vec<LayoutObj>
    ) -> LayoutObj;
}

pub fn panel<'window, 'ctx, T: Panel>(
    panel: &T,
    mut ctx: Context<'window, 'ctx>,
) -> LayoutObj {
    let mut children = Vec::new();

    let child_max_area = panel.open(ctx.max_area());
    ctx.open_socket(SocketName::default(), &mut children, child_max_area);

    panel.close(ctx, children)
}
