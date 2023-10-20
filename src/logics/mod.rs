pub mod parser;

#[derive(Debug, Clone, PartialEq)]
pub enum PTAtom {
    Cardinality(String, String),
    Fireability(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Formula {
    pub name: String,
    pub ty: FormulaTy,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FormulaTy {
    True,
    False,
    Prop(PTAtom),
    Neg(PTAtom),
    Not(Box<Self>),
    Or(Box<Self>, Box<Self>),
    And(Box<Self>, Box<Self>),
    Next(Box<Self>),
    Global(Box<Self>),
    Finally(Box<Self>),
    Until(Box<Self>, Box<Self>),
    Forall(Box<Self>),
    Exists(Box<Self>),
}