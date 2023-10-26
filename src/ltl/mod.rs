use crate::logics::*;
use crate::petri::*;

pub fn ltl_negate(input: FormulaTy) -> FormulaTy {
    // Negate an LTL formula
    match input {
        FormulaTy::True => FormulaTy::False,
        FormulaTy::False => FormulaTy::True,
        FormulaTy::Prop(atom) => FormulaTy::Neg(atom.clone()),
        FormulaTy::Neg(atom) => FormulaTy::Prop(atom.clone()),
        FormulaTy::Not(inner) => *inner.clone(),
        FormulaTy::Or(lhs, rhs) => FormulaTy::And(
            Box::new(ltl_negate(*lhs.clone())),
            Box::new(ltl_negate(*rhs.clone())),
        ),
        FormulaTy::And(lhs, rhs) => FormulaTy::Or(
            Box::new(ltl_negate(*lhs.clone())),
            Box::new(ltl_negate(*rhs.clone())),
        ),
        FormulaTy::Next(inner) => FormulaTy::Next(Box::new(ltl_negate(*inner.clone()))),
        FormulaTy::Global(inner) => FormulaTy::Finally(Box::new(ltl_negate(*inner.clone()))),
        FormulaTy::Finally(inner) => FormulaTy::Global(Box::new(ltl_negate(*inner.clone()))),
        FormulaTy::Until(lhs, rhs) => FormulaTy::Release(
            Box::new(ltl_negate(*lhs.clone())),
            Box::new(ltl_negate(*rhs.clone())),
        ),
        FormulaTy::Release(lhs, rhs) => FormulaTy::Until(
            Box::new(ltl_negate(*lhs.clone())),
            Box::new(ltl_negate(*rhs.clone())),
        ),
        FormulaTy::Forall(inner) => FormulaTy::Forall(Box::new(ltl_negate(*inner.clone()))),
        _ => input.clone(),
    }
}

pub fn ltl_simplify(input: FormulaTy) -> FormulaTy {
    match input {
        FormulaTy::Global(inner) => FormulaTy::Release(
            Box::new(FormulaTy::False),
            Box::new(*inner.clone()),
        ),
        FormulaTy::Finally(inner) => FormulaTy::Until(
            Box::new(FormulaTy::True),
            Box::new(*inner.clone()),
        ),
        FormulaTy::Not(inner) => FormulaTy::Not(Box::new(*inner.clone())),
        FormulaTy::Or(lhs, rhs) => FormulaTy::Or(
            Box::new(ltl_simplify(*lhs.clone())),
            Box::new(ltl_simplify(*rhs.clone())),
        ),
        FormulaTy::And(lhs, rhs) => FormulaTy::And(
            Box::new(ltl_simplify(*lhs.clone())),
            Box::new(ltl_simplify(*rhs.clone())),
        ),
        FormulaTy::Next(inner) => FormulaTy::Next(Box::new(ltl_negate(*inner.clone()))),
        FormulaTy::Until(lhs, rhs) => FormulaTy::Until(
            Box::new(ltl_simplify(*lhs.clone())),
            Box::new(ltl_simplify(*rhs.clone())),
        ),
        FormulaTy::Release(lhs, rhs) => FormulaTy::Release(
            Box::new(ltl_simplify(*lhs.clone())),
            Box::new(ltl_simplify(*rhs.clone())),
        ),
        FormulaTy::Forall(inner) => FormulaTy::Forall(Box::new(ltl_simplify(*inner.clone()))),
        _ => input.clone(),
    }
}