use std::collections::HashMap;

use indextree::{Arena, NodeId};
use shared::unique::Unique;

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
pub(crate) struct Forest<Id, Node>
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
    pub(crate) fn new() -> Self {
        Self {
            arena: Arena::new(),
            main_root: None,
            pending_root: None,
            id_to_node: HashMap::default(),
        }
    }

    pub(crate) fn set_pending_tree_root(&mut self, pending_tree_root: NodeId) {
        self.pending_root = Some(pending_tree_root);
    }

    pub(crate) fn create_node(&mut self, node: Node) -> NodeId {
        self.arena.new_node(node)
    }

    pub(crate) fn append_node(&mut self, parent: NodeId, child: NodeId) {
        parent.append(child, &mut self.arena);
    }

    pub(crate) fn make_relation(&mut self, id: Id, node_id: NodeId) {
        self.id_to_node.insert(id, node_id);
    }

    pub(crate) fn remove_relation(&mut self, id: Id) {
        self.id_to_node.remove(&id);
    }

    pub(crate) fn main_tree_root(&self) -> Option<&NodeId> {
        self.main_root.as_ref()
    }

    pub(crate) fn pending_tree_root(&self) -> Option<&NodeId> {
        self.pending_root.as_ref()
    }

    /// Promotes a pending tree root to main tree root.
    ///
    /// NOTE: this operation entirely destroys a main tree immediately, if pending tree root exists.
    /// So, be sure that all required operations are performed (about initialization,
    /// deinitialization and transferring responsibility to new nodes) before calling this method.
    pub(crate) fn promote_pending_tree_root(&mut self) {
        if let Some(root) = self.pending_root {
            if let Some(main_root) = self.main_root {
                main_root.remove_subtree(&mut self.arena);
            }

            self.main_root = Some(root);
            self.pending_root = None;
        }
    }

    pub(crate) fn node(&self, node_id: &NodeId) -> Option<&Node> {
        self.arena.get_data(*node_id)
    }

    pub(crate) fn node_mut(&mut self, node_id: &NodeId) -> Option<&mut Node> {
        self.arena.get_data_mut(*node_id)
    }

    pub(crate) fn node_by_id(&self, id: Id) -> Option<&Node> {
        self.id_to_node
            .get(&id)
            .and_then(|node_id| self.node(node_id))
    }

    pub(crate) fn node_by_id_mut(&mut self, id: Id) -> Option<Unique<Node>> {
        self.id_to_node.get(&id).and_then(|node_id| {
            self.arena
                .get_data_mut(*node_id)
                .map(|val| unsafe { Unique::from_mut(val) })
        })
    }

    pub(crate) fn parent_by_id(&self, id: Id) -> Option<&Node> {
        self.id_to_node
            .get(&id)
            .and_then(|node_id| node_id.parent(&self.arena))
            .and_then(|parent_node_id| self.node(&parent_node_id))
    }

    pub(crate) fn parent_by_id_mut(&mut self, id: Id) -> Option<Unique<Node>> {
        self.id_to_node
            .get(&id)
            .and_then(|node_id| node_id.parent(&self.arena))
            .and_then(|parent_node_id| {
                self.arena
                    .get_data_mut(parent_node_id)
                    .map(|val| unsafe { Unique::from_mut(val) })
            })
    }

    pub(crate) fn children_of(&self, node_id: &NodeId) -> indextree::Children<'_, Node> {
        node_id.children(&self.arena)
    }

    pub(crate) fn children_by_id(&self, id: Id) -> Vec<&Node> {
        self.id_to_node
            .get(&id)
            .map(|node_id| {
                self.children_of(node_id)
                    .flat_map(|child_node_id| self.node(&child_node_id))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub(crate) fn children_by_id_mut(&mut self, id: Id) -> Vec<Unique<Node>> {
        self.id_to_node
            .get(&id)
            .copied()
            .map(|node_id| {
                // INFO: here children twice collects into Vec<NodeId> and Vec<Unique<Node>> because
                // borrow checker doesn't allow immutable and mutable access to &mut self at the
                // same time.
                self.children_of(&node_id)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .flat_map(|child_node_id| {
                        self.arena
                            .get_data_mut(child_node_id)
                            .map(|val| unsafe { Unique::from_mut(val) })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub(crate) fn traverse(&self, node_id: &NodeId) -> indextree::Traverse<'_, Node> {
        node_id.traverse(&self.arena)
    }
}
