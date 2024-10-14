// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.
// Copyright Lucas Dietrich <ld.adecy@gmail.com> 2024

// Retrieved from https://github.com/tock/tock/blob/master/kernel/src/collections/list.rs
//! Linked list implementation.

use core::{cell::Cell, marker::PhantomData};

use super::Marker;

pub trait Node<'a, T: Node<'a, T, M>, M: Marker> {
    fn next(&'a self) -> &'a Link<'a, T, M>;
}

pub struct Link<'a, T: Node<'a, T, M>, M: Marker>(Cell<Option<&'a T>>, PhantomData<M>);

impl<'a, T: Node<'a, T, M>, M: Marker> Link<'a, T, M> {
    pub const fn empty() -> Self {
        Link(Cell::new(None), PhantomData)
    }
}

pub struct List<'a, T: Node<'a, T, M>, M: Marker> {
    head: Link<'a, T, M>,
    tail: Link<'a, T, M>,
}

impl<'a, T: Node<'a, T, M>, M: 'a + Marker> List<'a, T, M> {
    pub const fn empty() -> List<'a, T, M> {
        List {
            // head an tail must remain coherent together (either both None or both &Some)
            head: Link::empty(),
            tail: Link::empty(),
        }
    }

    pub fn push_front(&mut self, node: &'a T) {
        node.next().0.set(self.head.0.get());
        self.head.0.set(Some(node));
        if self.tail.0.get().is_none() {
            self.tail.0.set(Some(node));
        }
    }

    pub fn push_back(&mut self, node: &'a T) {
        node.next().0.set(None);
        if let Some(old_tail) = self.tail.0.get() {
            old_tail.next().0.set(Some(node));
        } else {
            self.head.0.set(Some(node));
        }
        self.tail.0.set(Some(node));
    }

    pub fn pop_head(&mut self) -> Option<&'a T> {
        self.head.0.get().map(|head| {
            let new_head = head.next().0.get();
            self.head.0.set(new_head);
            if new_head.is_none() {
                self.tail.0.set(None);
            }
            head
        })
    }

    pub fn iter(&self) -> ListIter<'a, T, M> {
        ListIter(self.head.0.get(), PhantomData)
    }

    // TODO: Evaluate the implementation of the remove method
    pub fn remove(&mut self, node: &'a T) {
        let mut prev: Option<&'a T> = None;
        for cur in self.iter() {
            if cur as *const T == node as *const T {
                let node_next = node.next().0.get();

                // If the searched node is not the first element of the list
                if let Some(prev) = prev {
                    prev.next().0.set(node_next);
                } else {
                    // prev is None, so the node to remove is the first item of the list
                    self.head.0.set(node_next);
                }

                // Update the tail, if we removed it
                if node_next.is_none() {
                    // If the node is also the first element of the list,
                    // prev is None, so tail becomes None also
                    self.tail.0.set(prev);
                }

                // remove done, return
                break;
            }

            prev = Some(cur)
        }
    }
}

pub struct ListIter<'a, T: Node<'a, T, M>, M: 'a + Marker>(Option<&'a T>, PhantomData<M>);

impl<'a, T: Node<'a, T, M>, M: Marker> Iterator for ListIter<'a, T, M> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.0.map(|cur| {
            self.0 = cur.next().0.get();
            cur
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SL;
    impl Marker for SL {}

    struct TestNode<'a> {
        val: u32,
        next: Link<'a, TestNode<'a>, SL>,
    }

    impl<'a> TestNode<'a> {
        pub const fn new(val: u32) -> Self {
            TestNode {
                val,
                next: Link::empty(),
            }
        }
    }

    impl<'a> Node<'a, TestNode<'a>, SL> for TestNode<'a> {
        fn next(&'a self) -> &'a Link<'a, TestNode<'a>, SL> {
            &self.next
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

        list.pop_head();
        let mut iter = list.iter();
        assert_eq!(iter.next().unwrap().val, 2);
        assert_eq!(iter.next().unwrap().val, 1);

        list.pop_head();
        let mut iter = list.iter();
        assert_eq!(iter.next().unwrap().val, 1);

        list.pop_head();
        let mut iter = list.iter();
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_push_back() {
        let mut list = List::empty();
        let node1 = TestNode::new(1);
        let node2 = TestNode::new(2);
        let node3 = TestNode::new(3);

        list.push_back(&node1);
        list.push_back(&node2);
        list.push_back(&node3);

        let mut iter = list.iter();
        assert_eq!(iter.next().unwrap().val, 1);
        assert_eq!(iter.next().unwrap().val, 2);
        assert_eq!(iter.next().unwrap().val, 3);
    }

    #[test]
    fn test_remove() {
        let mut list = List::empty();
        let node1 = TestNode::new(1);
        let node2 = TestNode::new(2);
        let node3 = TestNode::new(3);
        let node4 = TestNode::new(4);

        list.push_front(&node1);
        list.push_front(&node2);
        list.push_front(&node3);
        list.push_front(&node4);

        list.remove(&node2);
        let mut iter = list.iter();
        assert_eq!(iter.next().unwrap().val, 4);
        assert_eq!(iter.next().unwrap().val, 3);
        assert_eq!(iter.next().unwrap().val, 1);
        assert_eq!(list.head.0.get().unwrap().val, 4);
        assert_eq!(list.tail.0.get().unwrap().val, 1);

        list.remove(&node1);
        let mut iter = list.iter();
        assert_eq!(iter.next().unwrap().val, 4);
        assert_eq!(iter.next().unwrap().val, 3);
        assert_eq!(list.head.0.get().unwrap().val, 4);
        assert_eq!(list.tail.0.get().unwrap().val, 3);

        list.remove(&node4);
        let mut iter = list.iter();
        assert_eq!(iter.next().unwrap().val, 3);
        assert_eq!(list.head.0.get().unwrap().val, 3);
        assert_eq!(list.tail.0.get().unwrap().val, 3);

        list.remove(&node3);
        let mut iter = list.iter();
        assert!(iter.next().is_none());
        assert!(list.head.0.get().is_none());
        assert!(list.tail.0.get().is_none());
    }
}
