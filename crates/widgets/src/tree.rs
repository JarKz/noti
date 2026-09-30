use std::collections::HashMap;

use indextree::{Arena, NodeId};

/// A contract requires to retrieve some `T` from a type.
pub trait Get<T> {
    /// Retrieves a `T` value from a type.
    fn get(&self) -> T;
}

/// A contract requires to retrieve some `T` from a type if exists, otherwise none.
pub trait GetCarefully<T> {
    /// Retrieves a `Option<&T>` value from a type.
    fn get_carefully(&self) -> Option<&T>;
}

/// A forest of geeneric trees of nodes stored in an arena.
///
/// It allows fast use, retrieve, change and delete nodes within the same forest, managing complex
/// things internally and provides convenient API externally.
///
/// The important note that you must know is that a forest have a single main root, and each
/// operation where root doesn't mention implies main root. Otherwise explicitly mention an other
/// root.
pub struct Forest<Id, Node>
where
    Id: std::hash::Hash + Eq,
    Node: Get<Id>,
{
    arena: Arena<Node>,
    main_root: Option<NodeId>,
    pending_root: Option<NodeId>,
    id_to_node: HashMap<Id, NodeId>,
}

impl<Id, Node> Default for Forest<Id, Node>
where
    Id: std::hash::Hash + Eq,
    Node: Get<Id>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Id, Node> Forest<Id, Node>
where
    Id: std::hash::Hash + Eq,
    Node: Get<Id>,
{
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
            main_root: None,
            pending_root: None,
            id_to_node: HashMap::default(),
        }
    }

    /// Creates a new pending root that is not the main root. The pending tree can be used for a
    /// specific merge with the main tree.
    pub fn new_pending_root(&mut self, pending_root_node: Node)
    where
        Id: Clone,
    {
        let pending_root_id = pending_root_node.get();

        let pending_root_node_id = self.arena.new_node(pending_root_node);
        self.id_to_node
            .insert(pending_root_id.clone(), pending_root_node_id);
        self.pending_root = Some(pending_root_node_id);
    }

    /// Creates and attaches a node to a specific parent node.
    pub fn append_node(&mut self, parent: Id, node: Node) -> bool {
        let Some(parent_node_id) = self.id_to_node.get(&parent) else {
            return false;
        };

        let node_id = node.get();
        let child_node_id = self.arena.new_node(node);
        parent_node_id.append(child_node_id, &mut self.arena);
        self.id_to_node.insert(node_id, child_node_id);

        true
    }

    pub fn main_tree_root(&self) -> Option<&NodeId> {
        self.main_root.as_ref()
    }

    pub fn update_main_tree_root(&mut self, new_root: NodeId) {
        if let Some(main_root) = self.main_root {
            main_root.remove_subtree(&mut self.arena);
        }

        self.main_root = Some(new_root);
    }

    pub fn pending_tree_root(&self) -> Option<&NodeId> {
        self.pending_root.as_ref()
    }

    pub fn node(&self, node_id: &NodeId) -> Option<&Node> {
        self.arena.get_data(*node_id)
    }

    pub fn node_mut(&mut self, node_id: &NodeId) -> Option<&mut Node> {
        self.arena.get_data_mut(*node_id)
    }

    pub fn children_of(&self, node_id: &NodeId) -> indextree::Children<'_, Node> {
        node_id.children(&self.arena)
    }

    /// Checks whether a node is one of roots.
    fn is_root(&self, node_id: &NodeId) -> bool {
        self.main_root
            .as_ref()
            .is_some_and(|root_id| root_id == node_id)
            || self
                .pending_root
                .as_ref()
                .is_some_and(|pending_root_id| pending_root_id == node_id)
    }

    pub fn traverse(&self, node_id: &NodeId) -> indextree::Traverse<'_, Node> {
        node_id.traverse(&self.arena)
    }
}
