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
            $($($crate::engine::macros::hlist::ConsHack<
                $crate::engine::BTreeStorage<$id, $component>,)*)*
                $crate::engine::macros::hlist::Nil
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
                fn visit_component_types<
                    V: $crate::engine::VisitComponentTypes<Self, $id>>(
                        &self, v: &mut V)
                {
                    $(
                        v.visit::<$component>(self);
                    )*
                }

                #[allow(unused_variables)]
                fn visit_component_types_mut<
                    V: $crate::engine::VisitComponentTypesMut<Self, $id>>(
                        &mut self, v: &mut V)
                {
                    $(
                        v.visit_mut::<$component>(self);
                    )*
                }
            }

            $(
                impl $crate::engine::EntityComponent<$id, $component> for $name {
                    type Storage = $crate::engine::BTreeStorage<$id, $component>;

                    fn borrow(&self) -> &Self::Storage {
                        $crate::engine::macros::hlist::Get::get(&self.0)
                    }
                    fn borrow_mut(&mut self) -> &mut Self::Storage {
                        $crate::engine::macros::hlist::Get::get_mut(&mut self.0)
                    }
                }
            )*
        )*
    };
}
