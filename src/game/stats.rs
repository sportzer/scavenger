use ::engine::Component;

#[derive(Default)]
pub struct Damage(pub i8);
impl Component for Damage {}

#[derive(Default)]
pub struct Exhaustion(pub i8);
impl Component for Exhaustion {}

#[derive(Default)]
pub struct Hunger(pub i16);
impl Component for Hunger {}
