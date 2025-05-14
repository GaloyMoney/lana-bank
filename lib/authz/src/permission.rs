#[derive(Debug)]
pub struct Reference(pub &'static str);

#[derive(Debug)]
pub struct Action(pub &'static str);

#[derive(Debug)]
pub struct Object {
    pub name: &'static str,
    pub references: &'static [Reference],
    pub actions: &'static [Action],
}

#[derive(Debug)]
pub struct Module {
    pub name: &'static str,
    pub objects: &'static [Object],
}
