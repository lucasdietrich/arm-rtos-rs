/// This module contains the implementation of singly linked list and doubly
/// linked list data structures. They are not longer very efficient on modern
/// systems, and are not idiomatic at all in Rust, but whatever, they exist ...
pub mod doubly_linked;
pub mod singly_linked;

// This marker trait allow to define multiple implementation of the Node for the same structure
pub trait Marker {}
