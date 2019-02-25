use std::any::Any;
use crate::layout::Area;
use crate::element::{Id, Anchor, UIWidget, UIFilter, FilterStack, UISocket, UIRender, UIRenderImpl};
use super::state::{State, StateId, StateCache};

pub struct ContextData<'ui> {
    widget_id: Id,
    max_area: Area,
    next_state_id: StateId,
    prev_state_cache: &'ui StateCache,
    pub next_frame_filters: FilterStack,
    children: Vec<TreeNode<'static>>,
}

impl<'ui> ContextData<'ui> {
    pub fn new(
        widget_id: Id,
        max_area: Area,
        next_state_id: StateId,
        prev_state_cache: &'ui StateCache,
        children: Vec<TreeNode<'static>>,
    ) -> Self {
        ContextData {
            widget_id,
            max_area,
            next_state_id,
            prev_state_cache,
            next_frame_filters: FilterStack::default(),
            children,
        }
    }
}

pub struct TreeNode<'ctx> {
    pub kind: TreeNodeKind<'ctx>,
    pub target: Anchor,
    pub children: Vec<TreeNode<'ctx>>,
}

pub enum TreeNodeKind<'ctx> {
    Render(UIRender),
    Socket(UISocket<'ctx>),
    PreFilter(UIFilter),
    PostFilter(UIFilter),
    Widget(UIWidget),
}

pub struct Context<'ui, 'ctx> {
    data: &'ctx mut ContextData<'ui>,

    wip: Vec<TreeNode<'ctx>>,
    roots: Vec<TreeNode<'ctx>>,
}

impl<'ui, 'ctx> Context<'ui, 'ctx> {
    pub(super) fn new(
        data: &'ctx mut ContextData<'ui>,
    ) -> Self {
        Context {
            data,
            wip: Vec::new(),
            roots: Vec::new(),
        }
    }

    pub fn local<'super_ctx>(
        super_ctx: &'ctx mut Context<'ui, 'super_ctx>
    ) -> Self {
        Self::new(super_ctx.data)
    }

    pub fn widget_id(&self) -> Id {
        self.data.widget_id
    }

    pub fn max_area(&self) -> Area {
        self.data.max_area
    }

    // Pushes the given widget into the default socket ('')
    pub fn widget_begin(
        &mut self,
        widget: UIWidget,
    ) {
        self.widget_into_begin(Anchor::default(), widget)
    }

    pub fn widget_into_begin(
        &mut self,
        target: Anchor,
        widget: UIWidget,
    ) {
        let node = TreeNode {
            kind: TreeNodeKind::Widget(widget),
            target,
            children: Vec::new(),
        };

        self.begin(node);
    }

    pub fn socket_begin(
        &mut self,
        socket: UISocket<'ctx>,
    ) {
        let node = TreeNode {
            kind: TreeNodeKind::Socket(socket),
            target: Anchor::default(), // TODO: This shouldn't be here
            children: Vec::new(),
        };

        self.begin(node);
    }

    pub fn filter_pre_begin(
        &mut self,
        filter: UIFilter,
    ) {
        self.filter_pre_into_begin(filter, Anchor::default())
    }

    pub fn filter_pre_into_begin(
        &mut self,
        filter: UIFilter,
        target: Anchor, // TODO: This could be optional to allowing scattering filter to all targets
    ) {
        let node = TreeNode {
            kind: TreeNodeKind::PreFilter(filter),
            target,
            children: Vec::new(),
        };

        self.begin(node);
    }

    pub fn filter_post_begin(
        &mut self,
        filter: UIFilter,
    ) {
        self.filter_post_into_begin(filter, Anchor::default())
    }

    pub fn filter_post_into_begin(
        &mut self,
        filter: UIFilter,
        target: Anchor, //TODO: Same as above
    ) {
        let node = TreeNode {
            kind: TreeNodeKind::PostFilter(filter),
            target,
            children: Vec::new(),
        };

        self.begin(node);
    }

    pub fn children_all(
        &mut self,
    ) {
        // This absolutely should not need transmute.
        // We're converting Vec<TreeNode<'static>> into Vec<TreeNode<'ctx>>, which is perfectly fine...
        let children: &mut Vec<TreeNode<'ctx>> = unsafe { std::mem::transmute(&mut self.data.children) };
        self.roots.append(children);
    }

    pub fn children_into(
        &mut self,
        name: Anchor,
    ) {
        unimplemented!()
    }

    pub fn render_new(
        &mut self,
        min_area: Area,
        imp: Box<dyn UIRenderImpl>,
    ) {
        self.render(UIRender{
            min_area,
            imp,
        });
    }

    pub fn render(
        &mut self,
        render: UIRender,
    ) {
        self.render_into(Anchor::default(), render)
    }

    pub fn render_into(
        &mut self,
        target: Anchor,
        render: UIRender,
    ) {
        let node = TreeNode {
            kind: TreeNodeKind::Render(render),
            target,
            children: Vec::new(),
        };

        // Not calling 'begin' here, because renders can't have children
        self.roots.push(node);
    }

    fn begin(&mut self, mut node: TreeNode<'ctx>) {
        node.children = std::mem::replace(&mut self.roots, Vec::new());
        self.wip.push(node);
    }

    // Moves the context upward to the parent of the current widget
    // This will panic if the parent is not in scope for this context!
    pub fn end(&mut self) {
        let mut node = self.wip.pop().expect("Called 'end' after last widget in WIP stack");

        // While the node was in the WIP stack, it's children field actually held its siblings
        std::mem::swap(&mut self.roots, &mut node.children);
        self.roots.push(node);
    }

    pub fn await_sockets(
        self,
    ) {
        assert!(self.wip.is_empty(), "You cannot await while the WIP stack is not empty");

        // Need to create a new context
    }

    pub fn filter_pre_next_frame(
        &mut self,
        filter: UIFilter
    ) {
        self.data.next_frame_filters.filter_pre(filter);
    }

    pub fn filter_post_next_frame(
        &mut self,
        filter: UIFilter,
    ) {
        self.data.next_frame_filters.filter_post(filter);
    }

    pub fn new_state<T: Default + Clone + Send + Any>(&mut self) -> State<T> {
        let id = self.data.next_state_id.increment();
        State::new(id)
    }

    pub fn read_state<T: Default + Clone + Send + Any>(&self, state: State<T>) -> T {
        if state.id.frame_id != self.data.next_state_id.frame_id.prev() {
            panic!("Attempt to read state from wrong frame");
        }

        if let Some(v) = self.data.prev_state_cache.get(&state.id) {
            v.downcast_ref::<T>().expect("Mismatched types").clone()
        } else {
            Default::default()
        }
    }
}
