use std::mem;

use slab::Slab;

use crate::tree::{id::NodeId, node::Node};

pub mod id;
pub mod node;

pub struct Tree<T> {
    nodes: Slab<Node<T>>,
    generations: Vec<u32>,
    root: Option<NodeId>,
}

impl<T> Tree<T> {
    pub fn new() -> Self {
        Self {
            nodes: Slab::new(),
            generations: Vec::new(),
            root: None,
        }
    }

    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    pub fn create_root(&mut self, data: T) -> NodeId {
        let id = self.insert_node(None, data);
        self.root = Some(id);
        id
    }

    pub fn insert(&mut self, parent: NodeId, data: T) -> NodeId {
        let id = self.insert_node(Some(parent), data);
        self.nodes[parent.index as usize].children.push(id);
        id
    }

    pub fn insert_at(
        &mut self,
        parent: NodeId,
        position: usize,
        data: T,
    ) -> NodeId {
        let children_len = self.nodes[parent.index as usize].children.len();
        let position = position.min(children_len);
        let id = self.insert_node(Some(parent), data);
        self.nodes[parent.index as usize]
            .children
            .insert(position, id);
        id
    }

    pub fn get(&self, id: NodeId) -> Option<&T> {
        self.node(id).map(|node| &node.data)
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut T> {
        self.node_mut(id).map(|node| &mut node.data)
    }

    fn node(&self, id: NodeId) -> Option<&Node<T>> {
        self.nodes
            .get(id.index as usize)
            .filter(|node| node.generation == id.generation)
    }

    fn node_mut(&mut self, id: NodeId) -> Option<&mut Node<T>> {
        self.nodes
            .get_mut(id.index as usize)
            .filter(|node| node.generation == id.generation)
    }

    pub fn remove(&mut self, id: NodeId) -> T {
        let children = mem::take(&mut self.nodes[id.index as usize].children);
        for child in children {
            self.remove(child);
        }

        let node = self.nodes.remove(id.index as usize);
        self.generations[id.index as usize] =
            self.generations[id.index as usize].wrapping_add(1);

        if let Some(parent) = node.parent {
            self.nodes[parent.index as usize]
                .children
                .retain(|&child| child != id);
        }
        if self.root == Some(id) {
            self.root = None;
        }
        node.data
    }

    pub fn contains(&self, id: NodeId) -> bool {
        self.node(id).is_some()
    }

    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.node(id)?.parent
    }

    pub fn children(&self, id: NodeId) -> &[NodeId] {
        self.node(id).map(|n| n.children.as_slice()).unwrap_or(&[])
    }

    pub fn set_children(&mut self, parent: NodeId, children: Vec<NodeId>) {
        if self.nodes[parent.index as usize].children == children {
            return;
        }

        self.nodes[parent.index as usize].children = children;
        let child_count = self.nodes[parent.index as usize].children.len();
        for index in 0..child_count {
            let child = self.nodes[parent.index as usize].children[index];
            self.nodes[child.index as usize].parent = Some(parent);
        }
    }

    pub fn reorder_children(&mut self, parent: NodeId, order: &[NodeId]) {
        self.nodes[parent.index as usize].children = order.to_vec();
    }

    pub fn path_to_root(&self, id: NodeId) -> Vec<NodeId> {
        let mut path = vec![id];
        let mut current = id;
        while let Some(parent) = self.parent(current) {
            path.push(parent);
            current = parent;
        }
        path
    }

    pub fn depth(&self, id: NodeId) -> usize {
        self.node(id).map_or(0, |node| node.depth)
    }

    pub fn visit_subtree(&self, id: NodeId, mut f: impl FnMut(NodeId)) {
        let mut stack = vec![id];
        while let Some(current) = stack.pop() {
            f(current);
            if let Some(node) = self.node(current) {
                for &child in node.children.iter().rev() {
                    stack.push(child);
                }
            }
        }
    }

    pub fn subtree(&self, id: NodeId) -> Vec<NodeId> {
        let mut res = Vec::new();
        self.visit_subtree(id, |node| res.push(node));
        res
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    fn insert_node(&mut self, parent: Option<NodeId>, data: T) -> NodeId {
        let index = self.nodes.vacant_key();
        if index >= self.generations.len() {
            self.generations.resize(index + 1, 0);
        }
        let generation = self.generations[index];
        let id = NodeId {
            index: index as u32,
            generation,
        };

        self.nodes.insert(Node {
            generation,
            data,
            parent,
            depth: parent.map_or(0, |parent| self.depth(parent) + 1),
            children: Vec::new(),
        });

        id
    }
}
