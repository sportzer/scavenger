use std::collections::BTreeMap;
use std::collections::btree_map::{Keys, Iter, IterMut};

use super::*;

pub struct BTreeStorage<I: Id + Ord, C: Component>(BTreeMap<I, C>);

impl<I: Id + Ord, C: Component> Default for BTreeStorage<I, C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I: Id + Ord, C: Component> ComponentStorage<I, C> for BTreeStorage<I, C> {
    fn new() -> Self {
        BTreeStorage(BTreeMap::new())
    }

    fn has(&self, id: I) -> bool {
        self.0.contains_key(&id)
    }

    fn get(&self, id: I) -> Option<&C> {
        self.0.get(&id)
    }

    fn get_mut(&mut self, id: I) -> Option<&mut C> {
        self.0.get_mut(&id)
    }

    fn insert(&mut self, id: I, c: C) -> Option<C> {
        self.0.insert(id, c)
    }

    fn remove(&mut self, id: I) -> Option<C> {
        self.0.remove(&id)
    }

    fn get_or_else<F: FnOnce() -> C>(&mut self, id: I, f: F) -> &mut C {
        self.0.entry(id).or_insert_with(f)
    }

    fn count(&self) -> usize {
        self.0.len()
    }

    fn clear(&mut self) {
        self.0.clear()
    }

    // fn ids<'a>(&'a self) -> Box<Iterator<Item=I> + 'a> {
    //     Box::new(self.0.keys().map(|&id| id))
    // }

    // fn iter<'a>(&'a self) -> Box<Iterator<Item=(I, &C)> + 'a> {
    //     Box::new(self.0.iter().map(|(&id, c)| (id, c)))
    // }

    // fn iter_mut<'a>(&'a mut self) -> Box<Iterator<Item=(I, &mut C)> + 'a> {
    //     Box::new(self.0.iter_mut().map(|(&id, c)| (id, c)))
    // }
}

pub struct IdsMap<'a, I: Id + Ord + 'a, C: Component + 'a>(Keys<'a, I, C>);

impl<'a, I: Id + Ord + 'a, C: Component + 'a> Iterator for IdsMap<'a, I, C> {
    type Item = I;

    #[inline]
    fn next(&mut self) -> Option<I> {
        self.0.next().cloned()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn fold<Acc, F>(self, init: Acc, mut f: F) -> Acc
        where F: FnMut(Acc, Self::Item) -> Acc,
    {
        self.0.fold(init, move |acc, elt| f(acc, *elt))
    }
}

pub struct IterMap<'a, I: Id + Ord + 'a, C: Component + 'a>(Iter<'a, I, C>);

impl<'a, I: Id + Ord + 'a, C: Component + 'a> Iterator for IterMap<'a, I, C> {
    type Item = (I, &'a C);

    #[inline]
    fn next(&mut self) -> Option<(I, &'a C)> {
        self.0.next().map(|(&id, c)| (id, c))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn fold<Acc, F>(self, init: Acc, mut f: F) -> Acc
        where F: FnMut(Acc, Self::Item) -> Acc,
    {
        self.0.fold(init, move |acc, (&id, c)| f(acc, (id, c)))
    }
}

pub struct IterMutMap<'a, I: Id + Ord + 'a, C: Component + 'a>(IterMut<'a, I, C>);

impl<'a, I: Id + Ord + 'a, C: Component + 'a> Iterator for IterMutMap<'a, I, C> {
    type Item = (I, &'a mut C);

    #[inline]
    fn next(&mut self) -> Option<(I, &'a mut C)> {
        self.0.next().map(|(&id, c)| (id, c))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn fold<Acc, F>(self, init: Acc, mut f: F) -> Acc
        where F: FnMut(Acc, Self::Item) -> Acc,
    {
        self.0.fold(init, move |acc, (&id, c)| f(acc, (id, c)))
    }
}

impl<'a, I: Id + Ord + 'a, C: Component + 'a> Iterate<'a, I, C> for BTreeStorage<I, C> {
    type Ids = IdsMap<'a, I, C>;
    type Iter = IterMap<'a, I, C>;
    type IterMut = IterMutMap<'a, I, C>;

    fn ids(&'a self) -> Self::Ids {
        IdsMap(self.0.keys())
    }
    fn iter(&'a self) -> Self::Iter {
        IterMap(self.0.iter())
    }
    fn iter_mut(&'a mut self) -> Self::IterMut {
        IterMutMap(self.0.iter_mut())
    }
}
