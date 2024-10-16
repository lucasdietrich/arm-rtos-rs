// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.
// Copyright Lucas Dietrich <ld.adecy@gmail.com> 2024

// Retrieved from https://github.com/tock/tock/blob/master/kernel/src/collections/list.rs
//! Linked list implementation.

use core::{cell::Cell, marker::PhantomData};

// This marker trait allow to define multiple implementation of the Node for the same structure
pub trait Marker {}

pub trait Node<'a, T: Node<'a, T, M>, M: Marker> {
    fn next(&'a self) -> &'a Link<'a, T, M>;
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
}

pub struct Link<'a, T: Node<'a, T, M>, M: Marker>(Cell<Option<&'a T>>, PhantomData<M>);

impl<'a, T: Node<'a, T, M>, M: Marker> Link<'a, T, M> {
    pub const fn empty() -> Self {
        Link(Cell::new(None), PhantomData)
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
