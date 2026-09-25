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
    main_root: Id,
    pending_root: Option<Id>,
    detached_roots: Vec<Id>,
    id_to_node: HashMap<Id, NodeId>,
}

impl<Id, Node> Forest<Id, Node>
where
    Id: std::hash::Hash + Eq,
    Node: Get<Id>,
{
    /// Creates a new forest with a single main root node.
    pub fn new(root_node: Node) -> Self
    where
        Id: Clone,
    {
        let root_id = root_node.get();

        let mut arena = Arena::new();
        let root_node_id = arena.new_node(root_node);

        let mut id_to_node = HashMap::new();
        id_to_node.insert(root_id.clone(), root_node_id);

        Self {
            arena,
            main_root: root_id,
            pending_root: None,
            detached_roots: vec![],
            id_to_node,
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
        self.pending_root = Some(pending_root_id);
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

    /// Detaches a subtree by a particular node using its id. The subtree moves to detached trees
    /// with its root as the detached root.
    ///
    /// The operation cannot be applied to one of roots: main root, detached tree roots, pending
    /// tree roots. If provided node id is one of root, this method will silently do nothing.
    pub fn detach_subtree(&mut self, node_id: Id) {
        if self.is_root(&node_id) {
            return;
        }

        let Some(subtree_node_id) = self.id_to_node.get(&node_id) else {
            return;
        };

        subtree_node_id.detach(&mut self.arena);
        self.detached_roots.push(node_id);
    }

    /// Eventually removes and deletes a subtree by a particular node using its id. The subtree
    /// after this operation won't exist and it cannot be undone.
    ///
    /// The operation cannot be applied to one of roots: main root, detached tree roots, pending
    /// tree roots. If provided node id is one of root, this method will silently do nothing.
    pub fn remove_subtree(&mut self, node_id: Id) {
        if self.is_root(&node_id) {
            return;
        }

        let Some(subtree_node_id) = self.id_to_node.get(&node_id).cloned() else {
            return;
        };

        for node_edge in subtree_node_id.traverse(&self.arena) {
            match node_edge {
                indextree::NodeEdge::Start(node_id) => {
                    let node = self.arena.get_data(node_id).unwrap();
                    self.id_to_node.remove(&node.get());
                }
                indextree::NodeEdge::End(_) => (),
            }
        }

        subtree_node_id.remove_subtree(&mut self.arena);
    }

    /// Checks whether a node is one of roots.
    fn is_root(&self, node_id: &Id) -> bool {
        &self.main_root == node_id
            || self
                .pending_root
                .as_ref()
                .is_some_and(|pending_root_id| pending_root_id == node_id)
            || self.detached_roots.iter().any(|id| id == node_id)
    }
}
