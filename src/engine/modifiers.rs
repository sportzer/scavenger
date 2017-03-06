use std::ops::AddAssign;
use super::world::*;

pub trait Attribute: Copy {
    type Aggregator: Aggregator;

    fn new_aggregator(&self) -> Self::Aggregator;
}

pub trait BasicAttribute: Copy {
    type Aggregator: Aggregator + Default;
}

impl<A: BasicAttribute> Attribute for A {
    type Aggregator = A::Aggregator;

    fn new_aggregator(&self) -> Self::Aggregator {
        Default::default()
    }
}

pub trait Aggregator {
    type Input;
    type Output;

    fn add_modifier(&mut self, modifier: Self::Input);
    fn get_aggregate(self) -> Self::Output;
}

pub trait ModifyingComponent<A: Attribute, S: ComponentStorage<I, Self>, I: Id>: Component + Sized {
    fn get_modifiers<'a>(c: ComponentRef<'a, S, I, Self>, attr: A, aggregator: &'a mut A::Aggregator);
}

// TODO: do specialization inside of a private trait?
impl<C: Component, A: Attribute, S: ComponentStorage<I, C>, I: Id> ModifyingComponent<A, S, I> for C {
    default fn get_modifiers<'a>(_: ComponentRef<'a, S, I, Self>, _: A, _: &'a mut A::Aggregator) {}
}

pub trait EntityAttribute<S: EntityStorage<I>, I: Id>: Attribute {
    fn compute(self, s: &S, id: I) -> <Self::Aggregator as Aggregator>::Output {
        struct AggregationVisitor<A: Attribute, I: Id> {
            attr: A,
            id: I,
            aggregator: A::Aggregator,
        }
        let mut visitor = AggregationVisitor {
            attr: self,
            id: id,
            aggregator: self.new_aggregator(),
        };

        impl<A: Attribute, S: EntityStorage<I>, I: Id>
            VisitComponentTypes<S, I> for AggregationVisitor<A, I>
        {
            fn visit<C: Component>(&mut self, s: &S)
                where S: ComponentStorage<I, C>
            {
                if let Some(c) = s.get_ref(self.id) {
                    C::get_modifiers(c, self.attr, &mut self.aggregator);
                }
            }
        }

        s.visit_component_types(&mut visitor);
        visitor.aggregator.get_aggregate()
    }
}

impl<A: Attribute, S: EntityStorage<I>, I: Id> EntityAttribute<S, I> for A {}


// is AddAssign really the correct bound here?
#[derive(Default)]
pub struct SumAggregator<M: AddAssign>(M);

impl<M: AddAssign> Aggregator for SumAggregator<M> {
    type Input = M;
    type Output = M;

    fn add_modifier(&mut self, m: M) {
        self.0 += m;
    }

    fn get_aggregate(self) -> M { self.0 }
}

#[derive(Default)]
pub struct MaxAggregator<M: Ord>(Option<M>);

impl<M: Ord> Aggregator for MaxAggregator<M> {
    type Input = M;
    type Output = Option<M>;

    fn add_modifier(&mut self, m: M) {
        if self.0.as_ref().map(|old_m| old_m < &m).unwrap_or(true) {
            self.0 = Some(m);
        }
    }

    fn get_aggregate(self) -> Option<M> { self.0 }
}

#[derive(Default)]
pub struct MinAggregator<M: Ord>(Option<M>);

impl<M: Ord> Aggregator for MinAggregator<M> {
    type Input = M;
    type Output = Option<M>;

    fn add_modifier(&mut self, m: M) {
        if self.0.as_ref().map(|old_m| old_m > &m).unwrap_or(true) {
            self.0 = Some(m);
        }
    }

    fn get_aggregate(self) -> Option<M> { self.0 }
}
