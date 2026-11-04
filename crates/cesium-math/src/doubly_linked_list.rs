//! DoublyLinkedList matching CesiumJS `Core/DoublyLinkedList`.
//! Adapted from CesiumJS (Apache-2.0).

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

#[derive(Clone, Debug, PartialEq)]
pub struct DoublyLinkedListNode<T> {
    pub item: T,
    pub previous: Option<NodeId>,
    pub next: Option<NodeId>,
    pub id: NodeId,
    active: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DoublyLinkedList<T> {
    nodes: Vec<DoublyLinkedListNode<T>>,
    pub head: Option<NodeId>,
    pub tail: Option<NodeId>,
    len: usize,
}

impl<T> DoublyLinkedList<T> {
    pub const fn new() -> Self {
        Self {
            nodes: Vec::new(),
            head: None,
            tail: None,
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn node(&self, id: NodeId) -> Option<&DoublyLinkedListNode<T>> {
        self.nodes.get(id.0).filter(|n| n.active)
    }

    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut DoublyLinkedListNode<T>> {
        self.nodes.get_mut(id.0).filter(|n| n.active)
    }

    pub fn add(&mut self, item: T) -> NodeId {
        let new_id = NodeId(self.nodes.len());
        let node = DoublyLinkedListNode {
            item,
            previous: self.tail,
            next: None,
            id: new_id,
            active: true,
        };

        self.nodes.push(node);

        if let Some(old_tail) = self.tail {
            self.nodes[old_tail.0].next = Some(new_id);
            self.tail = Some(new_id);
        } else {
            self.head = Some(new_id);
            self.tail = Some(new_id);
        }

        self.len += 1;
        new_id
    }

    fn internal_remove(&mut self, id: NodeId) {
        let prev = self.nodes[id.0].previous;
        let next = self.nodes[id.0].next;

        if let Some(p) = prev {
            if let Some(n) = next {
                self.nodes[p.0].next = Some(n);
                self.nodes[n.0].previous = Some(p);
            } else {
                // Last node
                self.nodes[p.0].next = None;
                self.tail = Some(p);
            }
        } else if let Some(n) = next {
            // First node
            self.nodes[n.0].previous = None;
            self.head = Some(n);
        } else {
            // Only node
            self.head = None;
            self.tail = None;
        }

        self.nodes[id.0].next = None;
        self.nodes[id.0].previous = None;
    }

    pub fn remove(&mut self, id: NodeId) -> Option<T>
    where
        T: Default,
    {
        if id.0 >= self.nodes.len() || !self.nodes[id.0].active {
            return None;
        }

        self.internal_remove(id);
        self.nodes[id.0].active = false;
        self.len -= 1;

        Some(std::mem::take(&mut self.nodes[id.0].item))
    }

    pub fn splice(&mut self, node: NodeId, next_node: NodeId) {
        if node == next_node {
            return;
        }

        self.internal_remove(next_node);

        let old_node_next = self.nodes[node.0].next;
        self.nodes[node.0].next = Some(next_node);

        if self.tail == Some(node) {
            self.tail = Some(next_node);
        } else if let Some(nxt) = old_node_next {
            self.nodes[nxt.0].previous = Some(next_node);
        }

        self.nodes[next_node.0].next = old_node_next;
        self.nodes[next_node.0].previous = Some(node);
    }
}
