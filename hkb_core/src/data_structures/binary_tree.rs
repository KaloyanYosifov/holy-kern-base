use std::{cell::RefCell, rc::Rc};

trait Constraints: PartialEq + Eq + PartialOrd + Ord {}

impl<T: PartialEq + Eq + PartialOrd + Ord> Constraints for T {}

pub type NodeRef<T> = Rc<RefCell<Node<T>>>;

pub struct Node<T: PartialEq + Eq + PartialOrd + Ord> {
    pub val: T,
    left: Option<NodeRef<T>>,
    right: Option<NodeRef<T>>,
}

impl<T: PartialEq + Eq + PartialOrd + Ord> Node<T> {
    pub fn new(val: T, left_val: Option<T>, right_val: Option<T>) -> Self {
        let left = {
            if let Some(v) = left_val {
                Some(Rc::new(RefCell::new(Node::with_value(v))))
            } else {
                None
            }
        };
        let right = {
            if let Some(v) = right_val {
                Some(Rc::new(RefCell::new(Node::with_value(v))))
            } else {
                None
            }
        };

        Self { val, left, right }
    }

    pub fn with_value(val: T) -> Self {
        Self {
            val,
            left: None,
            right: None,
        }
    }

    pub fn with_nodes(val: T, left: Node<T>, right: Node<T>) -> Self {
        Self {
            val,
            left: Some(Rc::new(RefCell::new(left))),
            right: Some(Rc::new(RefCell::new(right))),
        }
    }

    pub fn get_left<'a>(&self) -> Option<NodeRef<T>> {
        if let Some(node) = self.left.as_ref() {
            Some(node.clone())
        } else {
            None
        }
    }

    pub fn get_right<'a>(&self) -> Option<NodeRef<T>> {
        if let Some(node) = self.right.as_ref() {
            Some(node.clone())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Node;

    #[test]
    fn it_can_create_a_node_with_value() {
        let node = Node::with_value(3);

        assert_eq!(3, node.val);
        assert!(matches!(node.get_left(), None));
        assert!(matches!(node.get_right(), None));
    }

    #[test]
    fn it_can_create_a_node_with_left_right_moved() {
        let node = Node::new(3, Some(5), Some(7));

        assert_eq!(3, node.val);
        assert_eq!(5, node.get_left().unwrap().borrow().val);
        assert_eq!(7, node.get_right().unwrap().borrow().val);
    }

    #[test]
    fn it_can_create_with_nodes_moved_directly() {
        let left_node = Node::with_value(5);
        let right_node = Node::with_value(7);
        let node = Node::with_nodes(3, left_node, right_node);

        assert_eq!(3, node.val);
        assert_eq!(5, node.get_left().unwrap().borrow().val);
        assert_eq!(7, node.get_right().unwrap().borrow().val);
    }
}
