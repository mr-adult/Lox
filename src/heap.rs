use std::{
    rc::Rc, 
    collections::HashSet
};

use crate::object::Object;

const NUM_INTERNAL_REFS: usize = 2;

pub (crate) struct ObjectHeap {
    first_obj: Option<Box<LLNode<Object>>>,
    objs: HashSet<Rc<Object>>
}

impl ObjectHeap {
    pub (crate) fn new() -> Self {
        Self {
            first_obj: None,
            objs: HashSet::new(),
        }
    }

    pub (crate) fn add(&mut self, obj: Object) -> Rc<Object> {
        let rc = Rc::new(obj);
        match self.objs.get(&*rc) {
            Some(obj) => obj.clone(),
            None => {
                self.objs.insert(rc.clone());
                self.first_obj = Some(Box::new(LLNode {
                    item: rc.clone(),
                    next: match std::mem::take(&mut self.first_obj) {
                        None => None,
                        Some(node) => Some(node),
                    }
                }));
                self.first_obj
                    .as_ref()
                    .expect("item we just inserted into the heap to still be there.")
                    .item
                    .clone()
            }
        }
    }

    pub (crate) fn collect_garbage(&mut self) {
        let mut current = &mut self.first_obj;
        loop {
            if let Some(current_ref) = &current {
                // If the only strong pointers are the ones in the heap, clean it up.
                // This has the side effect of moving us to the next node as well.
                if Rc::strong_count(&current_ref.item) == NUM_INTERNAL_REFS {
                    // borrow checker mad, just use unwrap()
                    *current = std::mem::take(
                        &mut current
                                .as_mut()
                                .unwrap()
                                .next
                    )
                // otherwise, continue the loop
                } else {
                    // borrow checker mad, just use unwrap()
                    current = &mut current
                            .as_mut()
                            .unwrap()
                            .next;
                }
            // If the current is None, we are done with the loop.
            } else {
                break;
            }
        }
    }
}

struct LLNode<T>
    where T: ?Sized {
    item: Rc<T>,
    next: Option<Box<LLNode<T>>>,
}

struct LLIter<'a, T> {
    next: Option<&'a LLNode<T>>,
}

impl<'a, T> Iterator for LLIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match std::mem::take(&mut self.next) {
            None => {
                return None;
            }
            Some(node) => {
                self.next = match node.next.as_ref() {
                    None => None,
                    Some(next) => Some(&**next),
                };
                return Some(&node.item);
            }
        }
    }
}

#[test]
fn garbage_collection() {
    let mut heap = ObjectHeap::new();
    for _ in 0..100 {
        heap.add(Object::String("test".into()));
    }
}