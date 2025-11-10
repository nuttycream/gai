// yes
//
// https://rust-leipzig.github.io/architecture/2016/12/20/idiomatic-trees-in-rust/

pub struct Arena<T> {
    nodes: Vec<Node<T>>,
}

pub struct Node<T> {
    parent: Option<usize>,
    children: Vec<usize>,

    pub data: T,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Arena { nodes: Vec::new() }
    }

    pub fn new_node(&mut self, value: T) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(Node {
            data: value,
            parent: None,
            children: Vec::new(),
        });
        idx
    }

    pub fn add_child(&mut self, parent_id: usize, child_id: usize) {
        self.nodes[parent_id].children.push(child_id);
        self.nodes[child_id].parent = Some(parent_id);
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}
