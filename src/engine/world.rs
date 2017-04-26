// TODO: switch to Option once ? works with that?
pub type QueryResult<T> = Result<T, ()>;

pub type ActionResult<T> = Result<T, ActionError>;

#[derive(Eq, PartialEq)]
pub enum ActionError {
    InCheckOnlyMode,
    Failed,
}

impl From<()> for ActionError {
    fn from(_: ()) -> ActionError { ActionError::Failed }
}

pub trait Component {}

pub trait World {
    fn new() -> Self;

    // TODO: move this to WorldRef/Mut
    fn err<T>(&self) -> ActionResult<T> {
        Err(ActionError::Failed)
    }
}

pub trait Id: Copy + Eq {}

// TODO: have WorldMut with mutation tracking
pub trait EntityStorage<I: Id>: World {
    fn visit_component_types<V: VisitComponentTypes<Self, I>>(&self, v: &mut V)
        where Self: Sized;
    fn visit_component_types_mut<V: VisitComponentTypesMut<Self, I>>(&mut self, v: &mut V)
        where Self: Sized;

    fn component<C: Component>(&self) -> &Self::Storage
        where Self: EntityComponent<I, C>
    {
        self.borrow()
    }
    fn component_mut<C: Component>(&mut self) -> &mut Self::Storage
        where Self: EntityComponent<I, C>
    {
        self.borrow_mut()
    }

    fn entity(&self, id: I) -> EntityRef<Self, I> where Self: Sized {
        EntityRef {
            world: self,
            id: id,
        }
    }

    fn entity_mut(&mut self, id: I) -> EntityMut<Self, I> where Self: Sized {
        EntityMut {
            world: self,
            id: id,
            committed: false,
            // check_only: false,
        }
    }
}

pub trait EntityComponent<I: Id, C: Component>: EntityStorage<I> {
    type Storage: ComponentStorage<I, C>;

    fn borrow(&self) -> &Self::Storage;
    fn borrow_mut(&mut self) -> &mut Self::Storage;
}


pub trait ComponentStorage<I: Id, C: Component> {
    fn new() -> Self;

    fn has(&self, id: I) -> bool;

    fn get(&self, id: I) -> Option<&C>;
    fn get_mut(&mut self, id: I) -> Option<&mut C>;
    fn insert(&mut self, id: I, c: C) -> Option<C>;
    fn remove(&mut self, id: I) -> Option<C>;

    fn get_or_else<F: FnOnce() -> C>(&mut self, id: I, f: F) -> &mut C;
    fn get_or_default(&mut self, id: I) -> &mut C where C: Default {
        self.get_or_else(id, Default::default)
    }

    fn count(&self) -> usize;
    fn clear(&mut self);

    // TODO: can I re-add these methods with appropriate Iterate bounds to get
    // better type inference?
    // fn ids<'a>(&'a self) -> Box<Iterator<Item=I> + 'a>;
    // fn iter<'a>(&'a self) -> Box<Iterator<Item=(I, &C)> + 'a>;
    // fn iter_mut<'a>(&'a mut self) -> Box<Iterator<Item=(I, &mut C)> + 'a>;
}

pub trait Iterate<'a, I, C: 'a> {
    type Ids: Iterator<Item=I> + 'a;
    type Iter: Iterator<Item=(I, &'a C)> + 'a;
    type IterMut: Iterator<Item=(I, &'a mut C)> + 'a;

    fn ids(&'a self) -> Self::Ids;
    fn iter(&'a self) -> Self::Iter;
    fn iter_mut(&'a mut self) -> Self::IterMut;
}


pub trait VisitComponentTypes<S: EntityStorage<I>, I: Id> {
    fn visit<C: Component>(&mut self, s: &S) where S: EntityComponent<I, C>;
}

pub trait VisitComponentTypesMut<S: EntityStorage<I>, I: Id> {
    fn visit_mut<C: Component>(&mut self, s: &mut S) where S: EntityComponent<I, C>;
}


// TODO: add mutation tracking so I can use CheckFailure in places
#[derive(Copy, Clone)]
pub struct EntityRef<'a, S: EntityStorage<I> + 'a, I: Id> {
    world: &'a S,
    id: I,
}

impl<'a, S: EntityStorage<I> + 'a, I: Id> EntityRef<'a, S, I> {
    pub fn has<C: Component>(&self) -> bool
        where S: EntityComponent<I, C>
    {
        self.world.component::<C>().has(self.id)
    }

    pub fn get<C: Component>(&self) -> QueryResult<&'a C>
        where S: EntityComponent<I, C>, S::Storage: 'a
    {
        self.world.component::<C>().get(self.id).ok_or(())
    }

    pub fn id(&self) -> I {
        self.id
    }

    pub fn world(&self) -> &'a S {
        &self.world
    }
}

enum CheckGuard {
    CheckOnly,
    Uncommitted,
    Committed,
}

// TODO: add mutation tracking so I can use CheckFailure in places
// TODO: make this a trait so check_only is implicit in the type?
pub struct EntityMut<'a, S: EntityStorage<I> + 'a, I: Id> {
    world: &'a mut S,
    // TODO: switch to using CheckGuard
    // check_guard: &'a mut CheckGuard,
    committed: bool,
    id: I,
}

// TODO: use check_only to deny mutation
impl<'a, S: EntityStorage<I> + 'a, I: Id> EntityMut<'a, S, I> {
    pub fn has<C: Component>(&self) -> bool
        where S: EntityComponent<I, C>
    {
        self.world.component::<C>().has(self.id)
    }

    pub fn get<C: Component>(&self) -> QueryResult<&C>
        where S: EntityComponent<I, C>, S::Storage: 'a
    {
        self.as_ref().get()
    }

    pub fn id(&self) -> I {
        self.id
    }

    pub fn world(&self) -> &S {
        &self.world
    }

    pub fn as_ref(&self) -> EntityRef<S, I> {
        EntityRef {
            world: self.world,
            id: self.id,
        }
    }

    pub fn get_mut<C: Component>(&mut self) -> ActionResult<&mut C>
        where S: EntityComponent<I, C>
    {
        match self.world.component_mut::<C>().get_mut(self.id) {
            Some(r) => {
                // TODO: figure out if there's a way to make commit() work
                // self.commit();
                self.committed = true;
                Ok(r)
            }
            None => Err(ActionError::Failed),
        }
    }

    pub fn insert<C: Component>(&mut self, c: C) -> Option<C>
        where S: EntityComponent<I, C>
    {
        self.commit();
        self.world.component_mut::<C>().insert(self.id, c)
    }

    pub fn remove<C: Component>(&mut self) -> ActionResult<C>
        where S: EntityComponent<I, C>
    {
        match self.world.component_mut::<C>().remove(self.id) {
            Some(r) => {
                self.commit();
                Ok(r)
            }
            None => Err(ActionError::Failed),
        }
    }

    pub fn get_or_else<C: Component, F: FnOnce() -> C>(&mut self, f: F) -> &mut C
        where S: EntityComponent<I, C>
    {
        self.commit();
        self.world.component_mut::<C>().get_or_else(self.id, f)
    }

    pub fn get_or_default<C: Component + Default>(&mut self) -> &mut C
        where S: EntityComponent<I, C>
    {
        self.commit();
        self.world.component_mut::<C>().get_or_default(self.id)
    }

    // TODO: return Result once we're using check_only mode
    pub fn commit(&mut self) {
        self.committed = true;
    }

    // pub fn world_mut(&mut self) -> &mut S {
    //     &mut self.world
    // }
}
