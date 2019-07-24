use crate::core::element::*;
use crate::core::filter::*;
use crate::state::*;
use crate::space::*;
use crate::util::linked_buffer::{LinkedBuffer, LBBox};
use crate::util::linked_queue::{QNode};

use super::children::{Children, ChildQNode};

pub struct Context<'slf, 'frm> {
    pub(crate) max_area: Area,
    pub(crate) children: Children<'frm>,

    pub(crate) prev_frame_state: &'slf StateCache,
    pub(crate) global_data: &'slf mut GlobalData,
    pub(crate) buffer: &'frm LinkedBuffer,
    pub(crate) subctx_stack: &'slf mut SubContextStack<'frm>,
}

pub(crate) struct ElementNode<'frm> {
    id: Id,
    elem: LBBox<'frm, dyn Element>,
    children: Children<'frm>,
}

impl<'frm> ElementNode<'frm> {
    pub(crate) fn new(id: Id, elem: LBBox<'frm, dyn Element>) -> Self {
        ElementNode {
            id,
            elem,
            children: Children::default(),
        }
    }
}

pub(crate) type SubContextStack<'frm> = Vec<(ChildQNode<'frm>, SocketName)>;

pub struct SubContext<'slf, 'ctx, 'frm> {
    pub(crate) max_area: Area,
    pub(crate) root: ElementNode<'frm>,
    pub(crate) ctx: &'slf mut Context<'ctx, 'frm>,
}

impl<'slf, 'ctx, 'frm> SubContext<'slf, 'ctx, 'frm> {
    pub fn close(mut self) -> LayoutNode<'frm> {
        while !self.ctx.subctx_stack.is_empty() {
            self.end();
        }

        // Need to create a context for the root element
        let ctx = Context {
            max_area: self.max_area,
            children: self.root.children.take(),

            prev_frame_state: self.ctx.prev_frame_state,
            global_data: self.ctx.global_data,
            buffer: self.ctx.buffer,
            subctx_stack: self.ctx.subctx_stack,
        };

        self.root.elem.run(ctx, self.root.id)
    }

    pub fn end(&mut self) -> &mut Self {
        let (node, socket) = self.ctx.subctx_stack.pop().expect("Bad call to 'end'");

        // Get the parent node
        let parent = match self.ctx.subctx_stack.last_mut() {
            Some(parent) => &mut parent.0,
            None => &mut self.root,
        };

        parent.children.get_or_create(socket).push_back_node(node);
        self
    }

    pub fn begin<'a, E: AllocElement<'frm>>(
        &'a mut self,
        socket: SocketName,
        id: Id,
        elem: E,
    ) -> &'a mut Self {
        // TODO(perf) - Could potentially do this as a single allocation
        let node = QNode::new(ElementNode::new(id, elem.alloc(self.ctx.buffer)));
        let node = self.ctx.buffer.alloc(node);

        self.ctx.subctx_stack.push((node, socket));
        self
    }

    pub fn connect_socket<'a>(
        &'a mut self,
        target: SocketName,
        socket: SocketName,
    ) -> &'a mut Self {
        // Get the current children
        let children = match self.ctx.children.remove(socket) {
            Some(children) => children,
            None => return self,
        };

        // Get the parent
        let parent = match self.ctx.subctx_stack.last_mut() {
            Some(parent) => &mut parent.0,
            None => &mut self.root,
        };

        // Insert the children into the parent
        parent.children.get_or_create(target).append(children);
        self
    }

    pub fn connect_all_sockets<'a>(
        &'a mut self,
    ) -> &'a mut Self {
        // Get the current children
        let children = self.ctx.children.take();

        // Get the parent
        let parent = match self.ctx.subctx_stack.last_mut() {
            Some(parent) => &mut parent.0,
            None => &mut self.root,
        };

        parent.children.append(children);

        self
    }

    pub fn new_state<T: StateT>(&mut self) -> State<T> {
        self.ctx.new_state()
    }

    pub fn read_state<T: StateT>(&self, state: State<T>) -> T {
        self.ctx.read_state(state)
    }
}

impl<'slf, 'frm> Context<'slf, 'frm> {
    pub fn max_area(&self) -> Area {
        self.max_area
    }

    // TODO: It would be nice if I didn't have to expose this
    pub fn buffer(&self) -> &'frm LinkedBuffer {
        self.buffer
    }

    pub fn open_element<'a, E: AllocElement<'frm>>(
        &'a mut self,
        max_area: Area,
        id: Id,
        elem: E,
    ) -> SubContext<'a, 'slf, 'frm> {
        let buf = self.buffer;

        // Clear the subcontext stack before using it
        self.subctx_stack.clear();

        SubContext {
            max_area,
            root: ElementNode::new(id, elem.alloc(buf)),
            ctx: self,
        }
    }

    pub fn open_socket<S: Socket<'frm>>(&mut self, name: SocketName, max_area: Area, socket: &mut S) {
        let children = match self.children.get(name) {
            Some(children) => children,
            None => return,
        };

        // Fill the socket
        while socket.remaining_capacity() != 0 {
            let mut child = match children.pop_front_node() {
                Some(child) => child,
                None => break,
            };

            // Run the child
            let sub_ctx = Context {
                max_area,
                children: child.children.take(),

                prev_frame_state: self.prev_frame_state,
                global_data: self.global_data,
                buffer: self.buffer,
                subctx_stack: self.subctx_stack,
            };

            socket.push(child.elem.run(sub_ctx, child.id));
        }
    }

    pub fn new_layout<L: Layout + 'frm>(&self, min_area: Area, layout: L) -> LayoutNode<'frm> {
        LayoutNode {
            min_area,
            layout: self.buffer.alloc(layout).unsize(),
        }
    }

    pub fn new_layout_null(&self) -> LayoutNode<'frm> {
        LayoutNode::null(self.buffer)
    }

    pub fn next_frame_pre_filter<F: Filter>(&mut self, _filter: F) {
        unimplemented!()
    }

    pub fn next_frame_post_filter<F: Filter>(&mut self, _filter: F) {
        unimplemented!()
    }

    pub fn new_state<T: StateT>(&mut self) -> State<T> {
        let id = self.global_data.next_state_id.increment();
        State::new(id)
    }

    pub fn read_state<T: StateT>(&self, state: State<T>) -> T {
        if state.id.frame_id != self.global_data.next_state_id.frame_id.prev() {
            panic!("Attempt to read state from wrong frame");
        }

        if let Some(v) = self.prev_frame_state.get(&state.id) {
            v.downcast_ref::<T>().expect("Mismatched types").clone()
        } else {
            Default::default()
        }
    }
}
