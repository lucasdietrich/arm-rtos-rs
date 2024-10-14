///! Doubly linked list implementation
///!
///! The major advantage of having a doubly linked list is the ability to remove
///! an element from the list in O(1) time complexity without actually knowing
///! the location of the whole list.
use core::{cell::Cell, marker::PhantomData};

use super::Marker;

pub trait Node<'a, T: 'a + Node<'a, T, M>, M: 'a + Marker> {
    fn next(&'a self) -> &'a Link<'a, T, M>;
    fn prev(&'a self) -> &'a Link<'a, T, M>;

    // Don't use
    fn remove(&'a self) {
        let prev = self.prev().0.get().unwrap();
        let next = self.next().0.get().unwrap();

        prev.next().0.set(Some(next));
        next.prev().0.set(Some(prev));
    }
}

/// The option is not required to node links, as they always point to a
/// real element, however on Link initialization we need to set the cell to
/// a valid &T. Unfortunately defining the link as bellow triggers the following
/// warning, and I'm not sure the implementation is correct.
///
/// Removing the Option<> will reduce the amount of .unwrap() and Some()
///
/// Warning:
/// ```
/// the type &T does not permit being left uninitialized
/// references must be non-null
/// #[warn(invalid_value)] on by defaultrustcClick for full compiler diagnostic
/// ```
///
/// Code illustration:
/// ```rust
/// Link(
///     Cell::new(unsafe { MaybeUninit::<&T>::uninit().assume_init() }),
///     PhantomData,
/// )
/// ```
pub struct Link<'a, T: Node<'a, T, M>, M: 'a + Marker>(Cell<Option<&'a T>>, PhantomData<M>);

impl<'a, T: Node<'a, T, M>, M: 'a + Marker> Link<'a, T, M> {
    pub const fn empty() -> Self {
        Link(Cell::new(None), PhantomData)
    }
}

fn insert_after<'a, T: Node<'a, T, M>, M: 'a + Marker>(node: &'a T, new: &'a T) {
    let old_next = node.next().0.get().unwrap();

    old_next.prev().0.set(Some(node));
    new.next().0.set(Some(old_next));

    node.next().0.set(Some(new));
    new.prev().0.set(Some(node));
}

fn insert_before<'a, T: Node<'a, T, M>, M: 'a + Marker>(node: &'a T, new: &'a T) {
    let old_prev = node.prev().0.get().unwrap();

    old_prev.next().0.set(Some(new));
    new.prev().0.set(Some(old_prev));

    node.prev().0.set(Some(new));
    new.next().0.set(Some(node));
}

pub struct List<'a, T: Node<'a, T, M>, M: 'a + Marker> {
    head: Cell<Option<&'a T>>,
    _marker: PhantomData<M>,
}

impl<'a, T: Node<'a, T, M>, M: 'a + Marker> List<'a, T, M> {
    pub const fn empty() -> Self {
        List {
            head: Cell::new(None),
            _marker: PhantomData,
        }
    }

    pub fn get_head(&self) -> Option<&'a T> {
        self.head.get()
    }

    pub fn get_tail(&self) -> Option<&'a T> {
        self.head.get().map(|head| head.prev().0.get().unwrap())
    }

    pub fn push_front(&mut self, node: &'a T) {
        if let Some(head) = self.head.get() {
            insert_before(head, node);
        } else {
            // Make new head point to itself
            node.next().0.set(Some(node));
            node.prev().0.set(Some(node));
        }

        // update head
        self.head.set(Some(node));
    }

    pub fn push_back(&mut self, node: &'a T) {
        if let Some(head) = self.head.get() {
            let tail = head.prev().0.get().unwrap();

            insert_before(tail, node);
        } else {
            // Make new head point to itself
            node.next().0.set(Some(node));
            node.prev().0.set(Some(node));

            // update head
            self.head.set(Some(node));
        }
    }

    pub fn pop_head(&mut self) -> Option<&'a T> {
        let head = self.head.get()?;
        let new_head = head.next().0.get().unwrap();

        if new_head as *const T == head as *const T {
            self.head.set(None);
        } else {
            head.remove();
            self.head.set(Some(new_head));
        }

        Some(head)
    }

    pub fn iter(&self) -> Iter<'a, T, M> {
        Iter {
            current: self.head.get(),
            head: self.head.get(),
            first_pass: true,
            _marker: PhantomData,
        }
    }
}

pub struct Iter<'a, T: Node<'a, T, M>, M: 'a + Marker> {
    current: Option<&'a T>,
    head: Option<&'a T>,
    first_pass: bool,
    _marker: PhantomData<M>,
}

impl<'a, T: Node<'a, T, M>, M: 'a + Marker> Iterator for Iter<'a, T, M> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        let curr = self.current?;

        // Compare references as raw pointers
        if curr as *const T == self.head.unwrap() as *const T {
            if self.first_pass {
                self.first_pass = false;
            } else {
                self.current = None;
                return None;
            }
        }

        // Go next
        self.current = Some(curr.next().0.get().unwrap());

        Some(curr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DL;
    impl Marker for DL {}

    struct TestNode<'a> {
        val: u32,
        next: Link<'a, TestNode<'a>, DL>,
        prev: Link<'a, TestNode<'a>, DL>,
    }

    impl<'a> TestNode<'a> {
        pub fn new(val: u32) -> Self {
            TestNode {
                val,
                next: Link::empty(),
                prev: Link::empty(),
            }
        }
    }

    impl<'a> Node<'a, TestNode<'a>, DL> for TestNode<'a> {
        fn next(&'a self) -> &'a Link<'a, TestNode<'a>, DL> {
            &self.next
        }

        fn prev(&'a self) -> &'a Link<'a, TestNode<'a>, DL> {
            &self.prev
        }
    }

    #[test]
    fn test_list() {
        let mut list = List::empty();
        let node1 = TestNode::new(1);
        let node2 = TestNode::new(2);
        let node3 = TestNode::new(3);

        list.push_front(&node1);
        list.push_front(&node2);
        list.push_front(&node3);

        let mut iter = list.iter();
        assert_eq!(iter.next().unwrap().val, 3);
        assert_eq!(iter.next().unwrap().val, 2);
        assert_eq!(iter.next().unwrap().val, 1);
        println!("head: {:?}", list.get_head().unwrap().val);

        let pop = list.pop_head();
        assert_eq!(pop.unwrap().val, 3);
        let mut iter = list.iter();
        assert_eq!(iter.next().unwrap().val, 2);
        assert_eq!(iter.next().unwrap().val, 1);
        println!("head: {:?}", list.get_head().unwrap().val);

        let pop = list.pop_head();
        assert_eq!(pop.unwrap().val, 2);
        let mut iter = list.iter();
        assert_eq!(iter.next().unwrap().val, 1);
        println!("head: {:?}", list.get_head().unwrap().val);

        let pop = list.pop_head();
        assert_eq!(pop.unwrap().val, 1);
        let mut iter = list.iter();
        assert!(iter.next().is_none());
    }
}
