use std::collections::HashMap;
use std::collections::HashSet;

pub mod parser;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PTAtom {
    Cardinality(String, String),
    Fireability(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Formula {
    pub name: String,
    pub ty: FormulaTy,
}

#[derive(Clone, PartialEq, Eq, Hash)]
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
    Release(Box<Self>, Box<Self>),
    Forall(Box<Self>),
    Exists(Box<Self>),
}

impl std::fmt::Debug for FormulaTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormulaTy::True => {
                write!(f, "True");
            },
            FormulaTy::False => {
                write!(f, "False");
            },
            FormulaTy::Prop(atom) => {
                match atom {
                    PTAtom::Cardinality(lhs, rhs) => {
                        write!(f, "C({} <= {})", lhs, rhs);
                    },
                    PTAtom::Fireability(inner) => {
                        write!(f, "F({})", inner);
                    },
                }
            },
            FormulaTy::Neg(atom) => {
                match atom {
                    PTAtom::Cardinality(lhs, rhs) => {
                        write!(f, "C({} <= {})", lhs, rhs);
                    },
                    PTAtom::Fireability(inner) => {
                        write!(f, "F({})", inner);
                    },
                }
            },
            FormulaTy::Not(inner) => {
                write!(f, "!");
                (*inner.clone()).fmt(f);
            },
            FormulaTy::Or(lhs, rhs) => {
                (*lhs.clone()).fmt(f);
                write!(f, " | ");
                (*rhs.clone()).fmt(f);
            },
            FormulaTy::And(lhs, rhs) => {
                (*lhs.clone()).fmt(f);
                write!(f, " & ");
                (*rhs.clone()).fmt(f);
            },
            FormulaTy::Finally(inner) => {
                write!(f, "F ");
                (*inner.clone()).fmt(f);
            },
            FormulaTy::Next(inner) => {
                write!(f, "X ");
                (*inner.clone()).fmt(f);
            },
            FormulaTy::Global(inner) => {
                write!(f, "G ");
                (*inner.clone()).fmt(f);
            },
            FormulaTy::Until(lhs, rhs) => {
                (*lhs.clone()).fmt(f);
                write!(f, " U ");
                (*rhs.clone()).fmt(f);
            },
            FormulaTy::Release(lhs, rhs) => {
                (*lhs.clone()).fmt(f);
                write!(f, " R ");
                (*rhs.clone()).fmt(f);
            },
            _ => {},
        }
        return Ok(());
    }
}

pub type FormulaSet = HashSet<FormulaTy>;