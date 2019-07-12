use std::collections::HashMap;
use std::collections::VecDeque;

use crate::core::element::*;
use crate::core::filter::*;
use crate::state::*;
use crate::space::*;

pub struct Context<'window> {
    pub(crate) max_area: Area,
    pub(crate) children: Children,

    pub(crate) prev_frame_state: &'window StateCache,
    pub(crate) global_data: &'window mut GlobalData,
}

pub(crate) struct Node {
    id: Id,
    elem: Box<dyn Element>,
    children: Children,
}

pub(crate) type Children = HashMap<SocketName, VecDeque<Node>>;

pub struct SubContext<'ctx, 'window> {
    pub(crate) ctx: &'ctx mut Context<'window>,
    pub(crate) max_area: Area,
    pub(crate) root: Node,
    pub(crate) stack: Vec<(Node, SocketName)>,
}

impl<'ctx, 'window> SubContext<'ctx, 'window> {
    pub fn close(mut self) -> LayoutObj {
        while !self.stack.is_empty() {
            self.end();
        }

        // Need to create a context for this element
        let sub_ctx = Context {
            max_area: self.max_area,
            children: self.root.children,

            prev_frame_state: self.ctx.prev_frame_state,
            global_data: self.ctx.global_data,
        };

        self.root.elem.run(sub_ctx, self.root.id)
    }

    pub fn end<'a>(&'a mut self) -> &'a mut SubContext<'ctx, 'window> {
        let (node, socket) = self.stack.pop().expect("Bad call to 'end'");

        // Get the parent node
        let parent = match self.stack.last_mut() {
            Some(parent) => &mut parent.0,
            None => &mut self.root,
        };

        parent.children.entry(socket).or_default().push_back(node);
        self
    }

    pub fn begin<'a, E: Element + 'static>(
        &'a mut self,
        socket: SocketName,
        id: Id,
        elem: E,
    ) -> &'a mut SubContext<'ctx, 'window> {
        let node = Node {
            id,
            elem: Box::new(elem),
            children: Children::new(),
        };

        self.stack.push((node, socket));
        self
    }

    pub fn connect_socket<'a>(
        &'a mut self,
        target: SocketName,
        socket: SocketName,
    ) -> &'a mut SubContext<'ctx, 'window> {
        // Get the current children
        let mut children = match self.ctx.children.remove_entry(&socket) {
            Some((_, children)) => children,
            None => return self,
        };

        // Get the parent
        let parent = match self.stack.last_mut() {
            Some(parent) => &mut parent.0,
            None => &mut self.root,
        };

        // Insert the children into the parent
        parent
            .children
            .entry(target)
            .or_default()
            .append(&mut children);
        self
    }

    pub fn new_state<T: StateT>(&mut self) -> State<T> {
        self.ctx.new_state()
    }

    pub fn read_state<T: StateT>(&self, state: State<T>) -> T {
        self.ctx.read_state(state)
    }
}

impl<'window> Context<'window> {
    pub fn max_area(&self) -> Area {
        self.max_area
    }

    pub fn open_element<'ctx, E: Element + 'static>(
        &'ctx mut self,
        max_area: Area,
        id: Id,
        elem: E,
    ) -> SubContext<'ctx, 'window> {
        SubContext {
            ctx: self,
            max_area,
            root: Node {
                id,
                elem: Box::new(elem),
                children: Children::new(),
            },
            stack: Vec::new(),
        }
    }

    pub fn open_socket(&mut self, name: SocketName, max_area: Area, socket: &mut dyn Socket) {
        let children = match self.children.get_mut(&name) {
            Some(children) => children,
            None => return,
        };

        // Fill the socket
        while socket.remaining_capacity() != 0 {
            let child = match children.pop_front() {
                Some(child) => child,
                None => break,
            };

            // Run the child
            let sub_ctx = Context {
                max_area,
                children: child.children,
                prev_frame_state: self.prev_frame_state,
                global_data: self.global_data,
            };

            socket.push(child.elem.run(sub_ctx, child.id));
        }
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
