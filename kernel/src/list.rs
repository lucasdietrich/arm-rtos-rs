// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.
// Copyright Lucas Dietrich <ld.adecy@gmail.com> 2024

// Retrieved from https://github.com/tock/tock/blob/master/kernel/src/collections/list.rs
//! Linked list implementation.

use core::cell::Cell;

pub trait Node<'a, T: Node<'a, T>> {
    fn next(&'a self) -> &'a Link<'a, T>;
}

pub struct List<'a, T: Node<'a, T>> {
    head: Link<'a, T>,
}

impl<'a, T: Node<'a, T>> List<'a, T> {
    pub const fn empty() -> List<'a, T> {
        List {
            head: Link::empty(),
        }
    }

    pub fn push_front(&mut self, node: &'a T) {
        node.next().0.set(self.head.0.get());
        self.head.0.set(Some(node))
    }

    pub fn pop_head(&mut self) -> Option<&'a T> {
        self.head.0.get().map(|head| {
            self.head.0.set(head.next().0.get());
            head
        })
    }

    pub fn iter(&self) -> ListIter<'a, T> {
        ListIter(self.head.0.get())
    }
}

pub struct Link<'a, T: Node<'a, T>>(Cell<Option<&'a T>>);

impl<'a, T: Node<'a, T>> Link<'a, T> {
    pub const fn empty() -> Self {
        Link(Cell::new(None))
    }
}

pub struct ListIter<'a, T: Node<'a, T>>(Option<&'a T>);

impl<'a, T: Node<'a, T>> Iterator for ListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.0.map(|cur| {
            self.0 = cur.next().0.get();
            cur
        })
    }
}
