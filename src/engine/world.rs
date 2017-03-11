use std::ops::Deref;

pub trait Component {}

pub trait World {
    fn new() -> Self;
}

// I'm not sure that Ord should be required here, but I need it for BTreeMap...
pub trait Id: Copy + Eq + Ord {}

pub trait EntityStorage<I: Id>: World {
    fn visit_component_types<V: VisitComponentTypes<Self, I>>(&self, v: &mut V) where Self: Sized;
    fn visit_component_types_mut<V: VisitComponentTypesMut<Self, I>>(&mut self, v: &mut V) where Self: Sized;

    fn has_component<C: Component>(&self, id: I) -> bool
        where Self: ComponentStorage<I, C>
    {
        self.has(id)
    }

    fn component_ids<'a, C: Component>(&'a self) -> Box<Iterator<Item=I> + 'a>
        where Self: ComponentStorage<I, C>
    {
        self.ids()
    }

    fn clear_component<C: Component>(&mut self)
        where Self: ComponentStorage<I, C>
    {
        self.clear()
    }

    fn entity_ref(&self, id: I) -> EntityRef<Self, I> where Self: Sized {
        EntityRef {
            world: self,
            id: id,
        }
    }

    fn entity_mut(&mut self, id: I) -> EntityMut<Self, I> where Self: Sized {
        EntityMut {
            world: self,
            id: id,
        }
    }
}

pub trait ComponentStorage<I: Id, C: Component>: EntityStorage<I> {
    fn has(&self, id: I) -> bool;

    fn get(&self, id: I) -> Option<&C>;
    fn get_mut(&mut self, id: I) -> Option<&mut C>;
    fn insert(&mut self, id: I, c: C) -> Option<C>;
    fn remove(&mut self, id: I) -> Option<C>;

    fn get_or_else<F: FnOnce() -> C>(&mut self, id: I, f: F) -> &mut C;
    fn get_or_default(&mut self, id: I) -> &mut C where C: Default {
        self.get_or_else(id, Default::default)
    }

    fn ids<'a>(&'a self) -> Box<Iterator<Item=I> + 'a>;
    fn iter<'a>(&'a self) -> Box<Iterator<Item=(I, &C)> + 'a>;
    fn iter_mut<'a>(&'a mut self) -> Box<Iterator<Item=(I, &mut C)> + 'a>;
    fn clear(&mut self);

    fn get_ref(&self, id: I) -> Option<ComponentRef<Self, I, C>> where Self: Sized {
        self.get(id).map(|c| ComponentRef {
            world: self,
            component: c,
            id: id,
        })
    }
}

pub trait VisitComponentTypes<S: EntityStorage<I>, I: Id> {
    fn visit<C: Component>(&mut self, s: &S) where S: ComponentStorage<I, C>;
}

pub trait VisitComponentTypesMut<S: EntityStorage<I>, I: Id> {
    fn visit_mut<C: Component>(&mut self, s: &mut S) where S: ComponentStorage<I, C>;
}

pub mod hlist {
    use std::marker::PhantomData;

    #[derive(Default, Debug)]
    pub struct Nil;

    #[derive(Default, Debug)]
    pub struct Cons<H, T>(H, T);

    pub trait Get<E, V> {
        fn get(&self) -> &E;
        fn get_mut(&mut self) -> &mut E;
    }

    impl<H, T> Get<H, Nil> for Cons<H, T> {
        fn get(&self) -> &H { &self.0 }
        fn get_mut(&mut self) -> &mut H { &mut self.0 }
    }

    impl<H, T, E, V> Get<E, Cons<H, V>> for Cons<H, T> where T: Get<E, V> {
        fn get(&self) -> &E { self.1.get() }
        fn get_mut(&mut self) -> &mut E { self.1.get_mut() }
    }

    // This is dumb, but is helpful for macros
    pub type ConsHack<H, T, I> = Cons<H, Cons<PhantomData<I>, T>>;
}

macro_rules! world {
    (
        $name:ident {
            $($id:ty: {
                $($component:ty,)*
            })*
        }
    ) => {
        // I'm using BTreeMap here so that iteration order will be predictable
        struct $name(
            $($($crate::engine::hlist::ConsHack<::std::collections::BTreeMap<$id, $component>,)*)*
                $crate::engine::hlist::Nil
                $($(, ($id, $component)>)*)*
        );

        impl $crate::engine::World for $name {
            fn new() -> $name {
                $name(Default::default())
            }
        }

        $(
            impl $crate::engine::EntityStorage<$id> for $name {
                #[allow(unused_variables)]
                fn visit_component_types<V: $crate::engine::VisitComponentTypes<Self, $id>>(&self, v: &mut V) {
                    $(
                        v.visit::<$component>(self);
                    )*
                }

                #[allow(unused_variables)]
                fn visit_component_types_mut<V: $crate::engine::VisitComponentTypesMut<Self, $id>>(&mut self, v: &mut V) {
                    $(
                        v.visit_mut::<$component>(self);
                    )*
                }
            }

            $(
                impl $crate::engine::ComponentStorage<$id, $component> for $name {
                    fn has(&self, id: $id) -> bool {
                        let storage: &::std::collections::BTreeMap<$id, $component> =
                            $crate::engine::hlist::Get::get(&self.0);
                        storage.contains_key(&id)
                    }

                    fn get(&self, id: $id) -> ::std::option::Option<&$component> {
                        let storage: &::std::collections::BTreeMap<$id, $component> =
                            $crate::engine::hlist::Get::get(&self.0);
                        storage.get(&id)
                    }

                    fn get_mut(&mut self, id: $id) -> ::std::option::Option<&mut $component> {
                        let storage: &mut ::std::collections::BTreeMap<$id, $component> =
                            $crate::engine::hlist::Get::get_mut(&mut self.0);
                        storage.get_mut(&id)
                    }

                    fn insert(&mut self, id: $id, c: $component) -> ::std::option::Option<$component> {
                        let storage: &mut ::std::collections::BTreeMap<$id, $component> =
                            $crate::engine::hlist::Get::get_mut(&mut self.0);
                        storage.insert(id, c)
                    }

                    fn remove(&mut self, id: $id) -> ::std::option::Option<$component> {
                        let storage: &mut ::std::collections::BTreeMap<$id, $component> =
                            $crate::engine::hlist::Get::get_mut(&mut self.0);
                        storage.remove(&id)
                    }

                    fn get_or_else<F: FnOnce() -> $component>(&mut self, id: $id, f: F) -> &mut $component {
                        let storage: &mut ::std::collections::BTreeMap<$id, $component> =
                            $crate::engine::hlist::Get::get_mut(&mut self.0);
                        storage.entry(id).or_insert_with(f)
                    }

                    fn ids<'a>(&'a self) -> Box<Iterator<Item=$id> + 'a> {
                        let storage: &::std::collections::BTreeMap<$id, $component> =
                            $crate::engine::hlist::Get::get(&self.0);
                        Box::new(storage.keys().map(|&id| id))
                    }

                    fn iter<'a>(&'a self) -> Box<Iterator<Item=($id, &$component)> + 'a> {
                        let storage: &::std::collections::BTreeMap<$id, $component> =
                            $crate::engine::hlist::Get::get(&self.0);
                        Box::new(storage.iter().map(|(&id, c)| (id, c)))
                    }

                    fn iter_mut<'a>(&'a mut self) -> Box<Iterator<Item=($id, &mut $component)> + 'a> {
                        let storage: &mut ::std::collections::BTreeMap<$id, $component> =
                            $crate::engine::hlist::Get::get_mut(&mut self.0);
                        Box::new(storage.iter_mut().map(|(&id, c)| (id, c)))
                    }

                    fn clear(&mut self) {
                        let storage: &mut ::std::collections::BTreeMap<$id, $component> =
                            $crate::engine::hlist::Get::get_mut(&mut self.0);
                        storage.clear()
                    }
                }
            )*
        )*
    };
}

#[derive(Copy, Clone)]
pub struct EntityRef<'a, S: EntityStorage<I> + 'a, I: Id> {
    world: &'a S,
    id: I,
}

impl<'a, S: EntityStorage<I> + 'a, I: Id> EntityRef<'a, S, I> {
    pub fn has_component<C: Component>(&self) -> bool where S: ComponentStorage<I, C> {
        self.world.has_component::<C>(self.id)
    }

    pub fn get<C: Component>(&self) -> Option<&'a C> where S: ComponentStorage<I, C> {
        self.world.get(self.id)
    }

    pub fn get_ref<C: Component>(&self) -> Option<ComponentRef<'a, S, I, C>> where S: ComponentStorage<I, C> {
        self.world.get(self.id).map(|c| ComponentRef {
            world: self.world,
            component: c,
            id: self.id,
        })
    }

    pub fn id(&self) -> I {
        self.id
    }

    pub fn world(&self) -> &'a S {
        &self.world
    }
}

pub struct EntityMut<'a, S: EntityStorage<I> + 'a, I: Id> {
    world: &'a mut S,
    id: I,
}

impl<'a, S: EntityStorage<I> + 'a, I: Id> EntityMut<'a, S, I> {
    pub fn has_component<C: Component>(&self) -> bool where S: ComponentStorage<I, C> {
        self.world.has_component::<C>(self.id)
    }

    pub fn get<C: Component>(&self) -> Option<&C> where S: ComponentStorage<I, C> {
        self.world.get(self.id)
    }

    pub fn get_ref<C: Component>(&self) -> Option<ComponentRef<S, I, C>> where S: ComponentStorage<I, C> {
        self.world.get(self.id).map(|c| ComponentRef {
            world: self.world,
            component: c,
            id: self.id,
        })
    }

    pub fn get_mut<C: Component>(&mut self) -> Option<&mut C> where S: ComponentStorage<I, C> {
        self.world.get_mut(self.id)
    }

    pub fn insert<C: Component>(&mut self, c: C) -> Option<C> where S: ComponentStorage<I, C> {
        self.world.insert(self.id, c)
    }

    pub fn remove<C: Component>(&mut self) -> Option<C> where S: ComponentStorage<I, C> {
        self.world.remove(self.id)
    }

    pub fn get_or_else<C: Component, F: FnOnce() -> C>(&mut self, f: F) -> &mut C where S: ComponentStorage<I, C> {
        self.world.get_or_else(self.id, f)
    }

    pub fn id(&self) -> I {
        self.id
    }

    pub fn world(&self) -> &S {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut S {
        &mut self.world
    }

    pub fn as_ref(&self) -> EntityRef<S, I> {
        EntityRef {
            world: self.world,
            id: self.id,
        }
    }
}

#[derive(Copy, Clone)]
pub struct ComponentRef<'a, S: ComponentStorage<I, C> + 'a, I: Id, C: Component + 'a> {
    world: &'a S,
    component: &'a C,
    id: I,
}

impl<'a, I: Id, C: Component + 'a, S: ComponentStorage<I, C> + 'a> ComponentRef<'a, S, I, C> {
    pub fn id(&self) -> I {
        self.id
    }

    pub fn world(&self) -> &'a S {
        &self.world
    }

    pub fn as_entity_ref(&self) -> EntityRef<'a, S, I> {
        EntityRef {
            world: self.world,
            id: self.id,
        }
    }
}

impl<'a, S: ComponentStorage<I, C> + 'a, I: Id, C: Component + 'a> Deref for ComponentRef<'a, S, I, C> {
    type Target = C;

    fn deref(&self) -> &C {
        &self.component
    }
}
