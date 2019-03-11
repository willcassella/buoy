use crate::prelude::*;
use crate::render::{CommandList, color};
use crate::render::commands::ColoredQuad;

use super::archetype;

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct SolidFill {
    pub color: color::RGBA8,
}

impl SolidFill {
    pub fn new(color: color::RGBA8) -> Self {
        SolidFill {
            color,
        }
    }

    fn generate_quad(color: color::RGBA8, region: Region, cmds: &mut CommandList) {
        cmds.add_colored_quads(&[ColoredQuad::new(From::from(region), color)]);
    }
}

impl archetype::Wrap for SolidFill {
    fn close_some(
        self,
        ctx: &mut Context,
        socket: &mut dyn Socket,
        child: LayoutObj,
    ) {
        let color = self.color;
        ctx.layout_new(socket, child.min_area, move |region: Region, cmds: &mut CommandList| {
            Self::generate_quad(color, region, cmds);
            child.imp.render(region, cmds);
        });
    }

    fn close_none(
        self,
        ctx: &mut Context,
        socket: &mut dyn Socket,
    ) {
        let color = self.color;
        ctx.layout_new(socket, Area::zero(), move |region: Region, cmds: &mut CommandList| {
            Self::generate_quad(color, region, cmds);
        });
    }
}

impl Element for SolidFill {
    type Suspended = ();

    fn run(
        self,
        ctx: &mut Context,
        socket: &mut dyn Socket,
    ) -> Option<Self::Suspended> {
        archetype::wrap(self, ctx, socket);
        None
    }
}
