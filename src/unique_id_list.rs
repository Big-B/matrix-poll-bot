use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::hash_map::Iter;

#[derive(Debug)]
pub struct UniqueIdList<T> {
    available: BTreeSet<usize>,
    entries: HashMap<usize, T>,
    max_entry: usize,
}

impl<T> UniqueIdList<T> {
    pub fn new() -> UniqueIdList<T> {
        UniqueIdList {
            available: BTreeSet::new(),
            entries: HashMap::new(),
            max_entry: 0,
        }
    }

    pub fn insert(&mut self, item: T) -> usize {
        if let Some(i) = self.available.pop_first() {
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
            // If the index we're removing is the last index, then we need to
            // pull it out of the list of available indexes and use the next
            // highest index
            if index == self.max_entry {
                self.available.pop_last();
                if let Some(x) = self.available.last() {
                    self.max_entry = *x;
                } else {
                    self.max_entry = 0;
                }
            } else {
                self.available.insert(index);
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


        assert_eq!(list.insert(0), 4);
        list.remove(3);
        assert_eq!(list.insert(0), 3);
    }
}
