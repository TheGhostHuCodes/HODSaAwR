use std::cell::RefCell;
use std::rc::Rc;

type Link = Option<Rc<RefCell<Node>>>;

#[derive(Debug, Clone)]
pub struct Node {
    value: String,
    next: Link,
}

impl Node {
    fn new(value: String) -> Rc<RefCell<Node>> {
        Rc::new(RefCell::new(Node { value, next: None }))
    }
}

#[derive(Debug)]
pub struct TransactionLog {
    head: Link,
    tail: Link,
    pub length: usize,
}

impl TransactionLog {
    pub fn new() -> TransactionLog {
        TransactionLog {
            head: None,
            tail: None,
            length: 0,
        }
    }

    /// Append a new value at the end of the `TransactionLog`.
    pub fn append(&mut self, value: String) {
        let new_node = Node::new(value);
        match self.tail.take() {
            // Go directly to the tail and add new_node to the next of the tail
            // node.
            Some(old_node) => old_node.borrow_mut().next = Some(new_node.clone()),
            // If tail is None, TransactionLog must have been empty so the head
            // must be None too. Assign new_node to head, assignment to tail
            // happens below.
            None => self.head = Some(new_node.clone()),
        };
        self.length += 1;
        // Always add new_node to the tail of the TransactionLog. That's the
        // whole purpose of append.
        self.tail = Some(new_node);
    }

    pub fn pop(&mut self) -> Option<String> {
        // Note `take()` returns an `Option<T>`, and calling `map()` on that
        // will map the supplied function over the inner T. The `Option` wrapper
        // will remain and be returned.
        self.head.take().map(|head_node| {
            // There is a head node, we borrow it and take the next node,
            // assigning it to the head field of `TransactionLog`.
            if let Some(next_node) = head_node.borrow_mut().next.take() {
                self.head = Some(next_node);
            // There is no head node, remove the `TransactionLog` tail as well
            // to create an empty `TransactionLog`.
            } else {
                self.tail.take();
            }
            self.length -= 1;
            // This should remove the `Rc`, resulting in a `RefCell`. Unless for
            // some reason something else was holding a reference to the head
            // node.
            Rc::try_unwrap(head_node)
                .ok()
                // Something else has a reference to the head node.
                .expect("Something is terribly wrong")
                // Remove the `RefCell`.
                .into_inner()
                .value
        })
    }
}

pub struct ListIterator {
    // Saves a reference to the current node.
    current_link: Link,
}

impl ListIterator {
    pub fn new(start_at: Link) -> ListIterator {
        ListIterator {
            current_link: start_at,
        }
    }
}

impl Iterator for ListIterator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let current_link = &self.current_link;
        let mut result = None;
        self.current_link = match current_link {
            // There is a current node.
            Some(current) => {
                let current_node = current.borrow();
                // Grab the current node value and return it as the result.
                result = Some(current_node.value.clone());
                // Update the `ListIterator.current_link` field with the next
                // node in the list.
                current_node.next.clone()
            }
            // There is no current node. We're done iterating.
            None => None,
        };

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_transaction_log_can_be_created() {
        let tl = TransactionLog::new();

        assert_eq!(tl.length, 0);
    }

    #[test]
    fn items_can_be_appended_and_popped_from_transaction_log() {
        let mut tl = TransactionLog::new();
        tl.append("Log Item 1".to_string());
        tl.append("Log Item 2".to_string());
        tl.append("Log Item 3".to_string());

        assert_eq!(tl.length, 3);
        assert_eq!(tl.pop(), Some("Log Item 1".to_string()));
        assert_eq!(tl.pop(), Some("Log Item 2".to_string()));
        assert_eq!(tl.pop(), Some("Log Item 3".to_string()));
        assert_eq!(tl.pop(), None);
    }

    #[test]
    fn transaction_log_can_be_forward_iterated() {
        let mut tl = TransactionLog::new();
        tl.append("Log Item 1".to_string());
        tl.append("Log Item 2".to_string());
        tl.append("Log Item 3".to_string());

        for t in ListIterator::new(tl.head.clone()).zip([
            "Log Item 1".to_string(),
            "Log Item 2".to_string(),
            "Log Item 3".to_string(),
        ]) {
            assert_eq!(t.0, t.1)
        }
    }
}
