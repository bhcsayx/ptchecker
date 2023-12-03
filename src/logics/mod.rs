use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::hash_set::Iter;
use std::hash::{Hash, Hasher};

pub mod parser;
pub mod ctl;
pub mod transys;

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
                        write!(f, "Card({} <= {})", lhs, rhs);
                    },
                    PTAtom::Fireability(inner) => {
                        write!(f, "Fire({})", inner);
                    },
                }
            },
            FormulaTy::Neg(atom) => {
                match atom {
                    PTAtom::Cardinality(lhs, rhs) => {
                        write!(f, "!Card({} <= {})", lhs, rhs);
                    },
                    PTAtom::Fireability(inner) => {
                        write!(f, "!Fire({})", inner);
                    },
                }
            },
            FormulaTy::Not(inner) => {
                write!(f, "!");
                (*inner.clone()).fmt(f);
            },
            FormulaTy::Or(lhs, rhs) => {
                write!(f, "(");
                (*lhs.clone()).fmt(f);
                write!(f, " | ");
                (*rhs.clone()).fmt(f);
                write!(f, ")");
            },
            FormulaTy::And(lhs, rhs) => {
                write!(f, "(");
                (*lhs.clone()).fmt(f);
                write!(f, " & ");
                (*rhs.clone()).fmt(f);
                write!(f, ")");
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
            FormulaTy::Forall(inner) => {
                write!(f, "A ");
                (*inner.clone()).fmt(f);
            },
            _ => {},
        }
        return Ok(());
    }
}

// Wrapper for hashset to implement hash thus enable storage by bimap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormulaSet {
    pub set: HashSet<FormulaTy>,
}

impl Hash for FormulaSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for elem in self.set.iter() {
            elem.hash(state);
        }
    }
}

impl FromIterator<FormulaTy> for FormulaSet {
    #[inline]
    fn from_iter<I: IntoIterator<Item = FormulaTy>>(iter: I) -> FormulaSet {
        let mut set: HashSet<FormulaTy> = HashSet::with_hasher(Default::default());
        set.extend(iter);
        FormulaSet {
            set: set,
        }
    }
}

impl FormulaSet {
    pub fn new() -> Self {
        FormulaSet {
            set: HashSet::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.set.len()
    }

    pub fn insert(&mut self, value: FormulaTy) {
        self.set.insert(value);
    }

    pub fn remove(&mut self, value: &FormulaTy) {
        self.set.remove(value);
    }

    pub fn contains(&self, value: &FormulaTy) -> bool {
        self.set.contains(value)
    }

    pub fn union(&self, other: &FormulaSet) -> FormulaSet {
        let mut res = HashSet::new();
        for i in self.set.union(&other.set) {
            res.insert(i.clone());
        }
        FormulaSet {
            set: res,
        }
    }

    pub fn intersection(&self, other: &FormulaSet) -> FormulaSet {
        let mut res = HashSet::new();
        for i in self.set.intersection(&other.set) {
            res.insert(i.clone());
        }
        FormulaSet {
            set: res,
        }
    }

    pub fn difference(&self, other: &FormulaSet) -> FormulaSet {
        let mut res = HashSet::new();
        for i in self.set.difference(&other.set) {
            res.insert(i.clone());
        }
        FormulaSet {
            set: res,
        }
    }
}