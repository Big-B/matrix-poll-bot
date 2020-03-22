use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::hash_map::Iter;
use std::cmp::Reverse;

#[derive(Debug)]
pub struct UniqueIdList<T> {
    available: BinaryHeap<Reverse<usize>>,
    entries: HashMap<usize, T>,
    max_entry: usize,
}

impl<T> UniqueIdList<T> {
    pub fn new() -> UniqueIdList<T> {
        UniqueIdList {
            available: BinaryHeap::new(),
            entries: HashMap::new(),
            max_entry: 0,
        }
    }

    pub fn insert(&mut self, item: T) -> usize {
        if let Some(Reverse(i)) = self.available.pop() {
            self.entries.insert(i, item);
            i
        } else {
            self.max_entry += 1;
            self.entries.insert(self.max_entry, item);
            self.max_entry
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if self.entries.contains_key(&index)
        {
            if index == self.max_entry {
                self.max_entry -= 1;
            } else {
                self.available.push(Reverse(index));
            }
            self.entries.remove(&index)
        } else {
            None
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.entries.get(&index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.entries.get_mut(&index)
    }

    pub fn iter(&self) -> Iter<usize, T> {
        self.entries.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let list = UniqueIdList::<usize>::new();
        assert_eq!(None, list.get(0));
    }

    #[test]
    fn test_incrementing_index() {
        let mut list = UniqueIdList::<usize>::new();
        assert_eq!(list.insert(0), 1);
        assert_eq!(list.insert(0), 2);
        assert_eq!(list.insert(0), 3);
    }

    #[test]
    fn test_lowest_available_index() {
        let mut list = UniqueIdList::<usize>::new();
        assert_eq!(list.insert(0), 1);
        assert_eq!(list.insert(0), 2);
        assert_eq!(list.insert(0), 3);

        list.remove(1);
        list.remove(2);
        assert_eq!(list.insert(0), 1);
        assert_eq!(list.insert(0), 2);
    }
}
