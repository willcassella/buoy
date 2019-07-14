use crate::prelude::*;
use crate::render::{
    commands::{HoverQuad, HoverState, Quad},
    CommandList,
};

#[derive(Clone)]
pub struct Hover {
    pub state: HoverState,
}

impl Hover {
    pub fn new(state: HoverState) -> Self {
        Hover {
            state,
        }
    }

    pub fn build(id: Id, state: HoverState) -> HoverBuilder {
        HoverBuilder {
            id,
            socket: SocketName::default(),
            element: Hover::new(state),
        }
    }
}

impl Element for Hover {
    fn run<'ctx, 'win>(&self, ctx: Context<'ctx, 'win>, _id: Id) -> LayoutNode<'win> {
        let state = self.state.clone();

        ctx.new_layout(
            Area::zero(),
            move |region: Region, cmds: &mut CommandList| {
                // Create the hover boundary
                let quad = HoverQuad {
                    quad: Quad::from(region),
                    state: state.clone(),
                };
                cmds.add_hover_quads(&[quad]);
            },
        )
    }
}

pub struct HoverBuilder {
    id: Id,
    socket: SocketName,
    element: Hover,
}

impl Builder for HoverBuilder {
    type Element = Hover;

    fn get_id(&self) -> Id {
        self.id
    }

    fn get_socket(&self) -> SocketName {
        self.socket
    }

    fn get_element(self) -> Self::Element {
        self.element
    }
}