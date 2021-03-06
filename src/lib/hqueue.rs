// Copyright (c) 2017 King's College London
// created by the Software Development Team <http://soft-dev.org/>
//
// The Universal Permissive License (UPL), Version 1.0
//
// Subject to the condition set forth below, permission is hereby granted to any
// person obtaining a copy of this software, associated documentation and/or
// data (collectively the "Software"), free of charge and under any and all
// copyright rights in the Software, and any and all patent rights owned or
// freely licensable by each licensor hereunder covering either (i) the
// unmodified Software as contributed to or provided by such licensor, or (ii)
// the Larger Works (as defined below), to deal in both
//
// (a) the Software, and
// (b) any piece of software and/or hardware listed in the lrgrwrks.txt file
// if one is included with the Software (each a "Larger Work" to which the Software
// is contributed by such licensors),
//
// without restriction, including without limitation the rights to copy, create
// derivative works of, display, perform, and distribute the Software and make,
// use, sell, offer for sale, import, export, have made, and have sold the
// Software and the Larger Work(s), and to sublicense the foregoing rights on
// either these or other terms.
//
// This license is subject to the following condition: The above copyright
// notice and either this complete permission notice or at a minimum a reference
// to the UPL must be included in all copies or substantial portions of the
// Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#![warn(missing_docs)]

use std::cmp::Ordering;
use std::fmt;

use ast::{Arena, DstNodeId, NodeId, SrcNodeId};

/// A `PriorityNodeId` wraps the height of a node with its id.
///
/// This type should be completely opaque to clients of this module.
/// Client code should construct a `HeightQueue` and call its methods,
/// which will return `NodeId`s directly, rather than the `PriorityNodeId`
/// wrapper.
#[derive(Clone, Eq, PartialEq)]
struct PriorityNodeId<U: PartialEq + Copy> {
    index: NodeId<U>,
    height: u32
}

impl<U: PartialEq + Copy> PriorityNodeId<U> {
    fn new(index: NodeId<U>, height: u32) -> PriorityNodeId<U> {
        PriorityNodeId { index, height }
    }

    fn id(&self) -> NodeId<U> {
        self.index
    }

    fn height(&self) -> u32 {
        self.height
    }
}

impl<U: Eq + PartialEq + Copy> Ord for PriorityNodeId<U> {
    fn cmp(&self, other: &PriorityNodeId<U>) -> Ordering {
        self.height.cmp(&other.height)
    }
}

impl<U: Eq + PartialEq + Copy> PartialOrd for PriorityNodeId<U> {
    fn partial_cmp(&self, other: &PriorityNodeId<U>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A queue of `NodeId`s sorted on the height of their respective nodes.
#[derive(Clone, Eq, PartialEq)]
pub struct HeightQueue<U: PartialEq + Copy> {
    queue: Vec<PriorityNodeId<U>> // Use Vec so we can call `sort()`.
}

impl<U: PartialEq + Copy> Default for HeightQueue<U> {
    fn default() -> HeightQueue<U> {
        HeightQueue { queue: vec![] }
    }
}

impl<U: fmt::Debug + PartialEq + Copy> fmt::Debug for HeightQueue<U> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[ ")?;
        for item in &self.queue {
            write!(f, "({:?}, {:?}) ", item.id(), item.height())?;
        }
        write!(f, "]")
    }
}

impl<U: PartialEq + Copy> HeightQueue<U> {
    /// Create empty priority queue.
    pub fn new() -> HeightQueue<U> {
        Default::default()
    }

    /// Remove (and discard) all items in this queue, leaving it empty.
    pub fn clear(&mut self) {
        self.queue.clear();
    }

    /// `true` if this queue is empty, `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Return the number of elements in this height queue.
    pub fn size(&self) -> usize {
        self.queue.len()
    }

    /// Get the id of the `Node` with the greatest height in the current queue.
    pub fn peek_max(&self) -> Option<u32> {
        if self.queue.is_empty() {
            return None;
        }
        Some(self.queue[self.queue.len() - 1].height)
    }

    /// Remove information about the tallest node(s) and return their `NodeId`.
    pub fn pop(&mut self) -> Vec<NodeId<U>> {
        let mut nodes = vec![];
        if self.is_empty() {
            return nodes;
        }
        let max = self.queue[self.queue.len() - 1].height;
        while !self.is_empty() && self.queue[self.queue.len() - 1].height == max {
            nodes.push(self.queue.pop().unwrap().id());
        }
        nodes
    }

    /// Push a new node into this priority queue, keeping the queue sorted.
    ///
    /// This method has no effect if the new node is already in the queue.
    pub fn push<T: Clone>(&mut self, index: NodeId<U>, arena: &Arena<T, U>) {
        let height = index.height(arena);
        let new_node = PriorityNodeId::new(index, height);
        if self.queue.contains(&new_node) {
            // Case 1: new node is already in the queue.
            return;
        } else if self.is_empty() || height <= self.queue[0].height() {
            // Case 2: new node is the shortest in the queue.
            self.queue.insert(0, new_node);
        } else if height >= self.queue[self.queue.len() - 1].height() {
            // Case 3: new node is the tallest in the queue.
            self.queue.push(new_node);
        } else {
            // Case 4: new node needs to be somewhere in the middle of the queue.
            for index in 0..self.queue.len() - 1 {
                if self.queue[index].height() <= height && self.queue[index + 1].height() > height {
                    self.queue.insert(index + 1, new_node);
                    return;
                }
            }
        }
    }

    /// Insert all the children of `parent` into this queue, keeping it sorted.
    pub fn push_children<T: Clone>(&mut self, parent: NodeId<U>, arena: &Arena<T, U>) {
        let children = parent.children(arena).collect::<Vec<NodeId<U>>>();
        for child in children {
            self.push(child, arena);
        }
    }

    /// Pop the top of the list and push the children of all of the tallest
    /// nodes back into the queue.
    pub fn pop_and_push_children<T: Clone>(&mut self,
                                           arena: &Arena<T, U>)
                                           -> Option<Vec<NodeId<U>>> {
        let tallest = self.pop();
        if !tallest.is_empty() {
            for node in &tallest {
                self.push_children(*node, arena);
            }
            return Some(tallest);
        }
        None
    }
}

/// Given two height queues, pop from each until they match in maximum height.
pub fn match_heights<T: PartialEq + Clone>(src_q: &mut HeightQueue<SrcNodeId>,
                                           src: &Arena<T, SrcNodeId>,
                                           dst_q: &mut HeightQueue<DstNodeId>,
                                           dst: &Arena<T, DstNodeId>) {
    while !src_q.is_empty()
          && !dst_q.is_empty()
          && src_q.peek_max().unwrap() != dst_q.peek_max().unwrap()
    {
        if src_q.peek_max().unwrap() > dst_q.peek_max().unwrap() {
            src_q.pop_and_push_children(&src);
        } else {
            dst_q.pop_and_push_children(&dst);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast::SrcNodeId;
    use test::Bencher;
    use test_common::{create_mult_arena, create_plus_arena};

    // Assert that `queue` is in sorted order and has the same size `arena`.
    fn assert_sorted<T: Clone + PartialEq>(queue: &HeightQueue<SrcNodeId>,
                                           arena: &Arena<T, SrcNodeId>) {
        let mut expected = arena.size();
        if expected == 0 {
            assert!(queue.is_empty());
            return;
        }
        let mut clone = queue.clone();
        let mut tallest: Vec<NodeId<SrcNodeId>>;
        loop {
            tallest = clone.pop();
            expected -= tallest.len();
            for node in &tallest {
                assert!(node.height(arena) == tallest[0].height(arena));
                if !clone.is_empty() {
                    assert!(node.height(arena) > clone.peek_max().unwrap());
                }
            }
            if clone.is_empty() {
                break;
            }
        }
        assert_eq!(0, expected);
    }

    #[test]
    fn clear() {
        let arena = create_mult_arena();
        let mut queue = arena.get_priority_queue();
        assert!(!queue.is_empty());
        queue.clear();
        assert!(queue.is_empty());
    }

    #[test]
    fn cmp_priority_node() {
        let p0 = PriorityNodeId::<SrcNodeId>::new(NodeId::new(0), 0);
        let p1 = PriorityNodeId::<SrcNodeId>::new(NodeId::new(0), 1);
        let p2 = PriorityNodeId::<SrcNodeId>::new(NodeId::new(0), 2);
        let p3 = PriorityNodeId::<SrcNodeId>::new(NodeId::new(0), 2);
        assert!(p0 < p1);
        assert!(p1 < p2);
        assert!(p2 == p3);
        assert!(p0 < p3);
        assert!(p3 > p1);
        assert!(p2 > p1);
        assert!(p3 > p0);
    }

    #[test]
    fn fmt_debug() {
        let arena = create_mult_arena();
        let queue = arena.get_priority_queue();
        let s = format!("{:?}", queue);
        // Three leaves in this arena can be placed in the queue in any order,
        // so we don't check the whole string, we just check the start of the
        // formatted string and the branch nodes at the end.
        let expected = " (NodeId { index: 2 }, 2) (NodeId { index: 0 }, 3) ]";
        assert_eq!("[ (NodeId { index:", s[..18].to_string());
        assert_eq!(expected, s[76..].to_string());
        assert_eq!(128, s.len());
    }

    #[test]
    fn new() {
        assert!(HeightQueue::<SrcNodeId>::new().is_empty());
    }

    #[test]
    fn push_children() {
        let arena = create_mult_arena();
        let mut queue = HeightQueue::<SrcNodeId>::new();
        queue.push_children(NodeId::new(0), &arena);
        let expected1 = vec![NodeId::new(2)]; // Expr *
        assert_eq!(expected1, queue.pop());
        let expected2 = vec![NodeId::new(1)]; // INT 1
        assert_eq!(expected2, queue.pop());
    }

    #[test]
    fn pop_and_push_children() {
        let arena = create_mult_arena();
        let mut queue = HeightQueue::<SrcNodeId>::new();
        assert!(queue.is_empty());
        queue.push(NodeId::new(0), &arena); // Root node.
        assert!(!queue.is_empty());
        assert!(queue.peek_max().is_some());
        assert_eq!(NodeId::new(0).height(&arena), queue.peek_max().unwrap());
        let tallest_wrapped = queue.pop_and_push_children(&arena);
        assert!(tallest_wrapped.is_some());
        let tallest = tallest_wrapped.unwrap();
        assert_eq!(1, tallest.len());
        assert_eq!(NodeId::new(0), tallest[0]);
        assert_eq!(NodeId::new(0).children(&arena)
                                 .collect::<Vec<NodeId<SrcNodeId>>>()
                                 .len(),
                   queue.size());
    }

    #[test]
    fn peek_max() {
        let arena = create_mult_arena();
        let queue = arena.get_priority_queue();
        let height = queue.peek_max().unwrap();
        assert_eq!(NodeId::new(0).height(&arena), height);
    }

    #[test]
    fn pop() {
        let arena = create_mult_arena();
        let mut queue = arena.get_priority_queue();
        assert_eq!(vec![NodeId::new(0)], queue.pop());
        assert_eq!(vec![NodeId::new(2)], queue.pop());
        // Nodes 1, 3, 4 have the same height, and so may be stored in any order.
        let expected = vec![NodeId::new(1), NodeId::new(3), NodeId::new(4)];
        let leaves = queue.pop();
        assert_eq!(expected.len(), leaves.len());
        for leaf in leaves {
            assert!(expected.contains(&leaf));
        }
        assert!(queue.is_empty());
    }

    #[test]
    fn push() {
        let arena = create_mult_arena();
        let queue = arena.get_priority_queue();
        assert_sorted(&queue, &arena);
    }

    #[test]
    fn push_identical_nodes() {
        let arena = create_mult_arena();
        let mut queue = HeightQueue::new();
        queue.push(NodeId::new(0), &arena);
        let formatted = format!("{:?}", queue);
        let expected = "[ (NodeId { index: 0 }, 3) ]";
        assert_eq!(expected, formatted);
        queue.push(NodeId::new(0), &arena); // Should have no effect.
        assert_eq!(expected, formatted);
    }

    #[test]
    fn test_match_heights() {
        let plus = create_plus_arena();
        let mult = Arena::<String, DstNodeId>::from(create_mult_arena());
        let mut plus_q: HeightQueue<SrcNodeId> = HeightQueue::new();
        let mut mult_q: HeightQueue<DstNodeId> = HeightQueue::new();
        assert!(plus_q.is_empty());
        assert!(mult_q.is_empty());
        for node in NodeId::new(0).breadth_first_traversal(&plus) {
            plus_q.push(node, &plus);
        }
        for node in NodeId::new(0).breadth_first_traversal(&mult) {
            mult_q.push(node, &mult);
        }
        assert_eq!(2, NodeId::new(0).height(&plus));
        assert_eq!(3, NodeId::new(0).height(&mult));
        match_heights(&mut plus_q, &plus, &mut mult_q, &mult);
        assert_eq!(plus_q.peek_max().unwrap(), mult_q.peek_max().unwrap());
        assert_eq!(2, plus_q.peek_max().unwrap());
        assert_eq!(2, mult_q.peek_max().unwrap());
    }

    const BENCH_ITER: usize = 10000;

    #[bench]
    fn bench_push(bencher: &mut Bencher) {
        let mut arena: Arena<&str, SrcNodeId> = Arena::new();
        for _ in 0..BENCH_ITER {
            arena.new_node("", String::from(""), None, None, None, None);
        }
        let mut queue = HeightQueue::new();
        // Because `HeightQueues` are sets, each iteration of this
        // microbenchmark must push a distinct `NodeId` to the queue, to avoid
        // the optimisation that does not attempt to push an existing value to
        // the structure.
        bencher.iter(|| {
                         for id in 0..BENCH_ITER {
                             queue.push(NodeId::new(id), &arena);
                             queue.clear();
                         }
                     });
    }
}
