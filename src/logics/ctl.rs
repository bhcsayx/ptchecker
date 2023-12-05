use std::collections::{HashSet, HashMap};
use crate::logics::{FormulaTy, PTAtom};
use crate::logics::transys::{State, TranSys};


pub fn visit_eu(T: &TranSys, s: State, f: FormulaTy, info: &mut HashMap<(State, FormulaTy), bool>, marks: & mut HashSet<State>) {
    marks.insert(s);
    let fc = f.clone();
    if info.contains_key(&(s, f.clone())) {
        if let Some(true) = info.get(&(s, f.clone())) {
            info.insert((s, f), true);
        }
    } else if let FormulaTy::Until(f1, f2) = fc {
        check(T, s, *f2.clone(), info);
        if let Some(true) = info.get(&(s, *f2.clone())) {
            info.insert((s, f.clone()), true);
        }

        check(T, s,*f1.clone(), info);
        if let Some(false) = info.get(&(s, *f1.clone())) {
            return;
        }

        if let Some(ref set) = T.states_from(&s) {
            for s1 in &**set {
                if ! marks.contains(&s1) {
                    visit_eu(T, *s1, f.clone(), info, marks);
                }
            }
        }
    }
}

pub fn checkEU(T: &TranSys, sr: State, f: FormulaTy, info: &mut HashMap<(State, FormulaTy), bool>, marks: & mut HashSet<State>) {
    visit_eu(T, sr, f, info, marks);
}

pub fn visit_au(T: &TranSys, s: State, f: FormulaTy, info: &mut HashMap<(State, FormulaTy), bool>, cp: &mut Vec<State>) {
    let fc = f.clone();
    if info.contains_key(&(s, f.clone())) {
        if let Some(false) = info.get(&(s, f.clone())) {
            for s1 in cp.iter() {
                info.insert((*s1, f.clone()), false);
            }
        }
    } else if let FormulaTy::Forall(boxed_formula) = fc {
        if let FormulaTy::Until(f1, f2) = *boxed_formula {
            check(T, s,*f2.clone(), info);
            if let Some(true) = info.get(&(s, *f2.clone())) {
                info.insert((s, f.clone()), true);
            }

            check(T, s,*f1.clone(), info);
            if let Some(false) = info.get(&(s,*f1.clone())) {
                info.insert((s, f.clone()), false);
                for s1 in cp.iter() {
                    info.insert((*s1, f.clone()), false);
                }
            }

            cp.push(s);
            if let Some(&ref set) = T.states_from(&s) {
                for s1 in set {
                    if !cp.contains(&s1) {
                        visit_au(T, *s1, f.clone(), info, cp);
                    } else {
                        for s1 in cp.iter() {
                            info.insert((*s1, f.clone()), false);
                        }
                    }
                }
            }
            info.insert((s, f), true);
            cp.pop();
        }
    }
}


pub fn checkAU(T: &TranSys, sr: State, f: FormulaTy, info: &mut HashMap<(State, FormulaTy), bool>) {
    let mut cp = Vec::new();
    visit_au(T, sr, f, info, &mut cp);
}

pub fn simplify(f: FormulaTy) -> FormulaTy {
    match f {
        FormulaTy::Global(f1) => {
            return FormulaTy::Until(Box::new(FormulaTy::False), f1);
        }
        FormulaTy::Finally(f1) => {
            return FormulaTy::Until(Box::new(FormulaTy::True), f1);
        }
        _ => return f
    }
}

pub fn check(T: &TranSys, s: State, f: FormulaTy, info: &mut HashMap<(State, FormulaTy), bool>) {
    let fc = f.clone();
    if let None = info.get(&(s, f.clone())) {
        match fc {
            FormulaTy:: True => {
                info.insert((s, f.clone()), true);
            }
            FormulaTy:: False => {
                info.insert((s, f.clone()), false);
            }
            FormulaTy::Prop(f1) => {
                let name = match f1 {
                    PTAtom::Fireability(name) => {name}
                    _ => { panic!("Cardinality not supported") }
                };
                if let Some(&ref set) = T.label_of(&name) {
                    info.insert((s, f),
                                if set.contains(&s) { true } else { false }
                    );
                }
            }
            FormulaTy::Not(f1) => {
                check(T, s, *f1.clone(), info);
                if let Some(&b) = info.get(&(s,*f1.clone())) {
                    info.insert((s, f), b);
                }
            }
            FormulaTy::Or(f1, f2) => {
                check(T, s, *f1.clone(), info);
                if let Some(&b) = info.get(&(s,*f1.clone())) {
                    if b {
                        info.insert((s, f), true);
                    } else {
                        check(T, s, *f2.clone(), info);
                        if let Some(&b) = info.get(&(s, *f2)) {
                            info.insert((s, f), b);
                        }
                    }
                }
            }
            FormulaTy::And(f1, f2) => {
                check(T, s, *f1.clone(), info);
                if let Some(&b) = info.get(&(s,*f1.clone())) {
                    if b {
                        check(T, s, *f2.clone(), info);
                        if let Some(&b) = info.get(&(s, *f2)) {
                            info.insert((s, f), b);
                        }
                    } else {
                        info.insert((s, f), false);
                    }
                }
            }
            FormulaTy::Next(f1) => {
                if let Some(&ref set) = T.states_from(&s) {
                    for s1 in set {
                        check(T, *s1, *f1.clone(), info);
                        if let Some(&b) = info.get(&(*s1, *f1.clone())) {
                            info.insert((s, f.clone()), true);
                            return;
                        }
                    }
                }
                info.insert((s, f), false);
            }
            // FormulaTy::Finally(f1) => {
            //     check(T, s, simplify(f.clone()), info);
            // }
            // FormulaTy::Global(f1) => {
            //     check(T, s, simplify(f.clone()), info);
            // }
            FormulaTy::Forall(f1) => {
                match *f1 {
                    FormulaTy::Next(f2) => {
                        if let Some(&ref set) = T.states_from(&s) {
                            for s1 in set {
                                check(T, *s1,*f2.clone(), info);
                                if let Some(false) = info.get(&(*s1, *f2.clone())) {
                                    info.insert((s, f), false);
                                    return;
                                }
                            }
                            info.insert((s, f.clone()), true);
                        }
                    }
                    FormulaTy::Global(_) | FormulaTy::Finally(_) => {
                        let f2 = simplify(*f1.clone());
                        checkAU(T, s, FormulaTy::Forall(Box::new(f2)), info);
                    }
                    _ => {}
                }
            }
            FormulaTy::Exists(f1) => {
                match *f1 {
                    FormulaTy::Next(f2) => {
                        if let Some(&ref set) = T.states_from(&s) {
                            for s1 in set {
                                check(T, *s1,*f2.clone(), info);
                                if let Some(true) = info.get(&(*s1, *f2.clone())) {
                                    info.insert((s, f), true);
                                    return;
                                }
                            }
                        }
                        info.insert((s, f), false);
                    }
                    FormulaTy::Global(_) | FormulaTy::Finally(_) => {
                        let f2 = simplify(*f1.clone());
                        let mut marks = HashSet::new();
                        checkEU(T, s, FormulaTy::Exists(Box::new(f2)), info, &mut marks);
                    }
                    _ => {}
                }
            }
            _ => {
                println!("Malformed term {:?}", fc);
            }
        }
    }
}

pub fn almc(T: TranSys, s: State, f: FormulaTy) -> bool {
    let mut info: HashMap<(State, FormulaTy), bool> = HashMap::new();
    check(&T, s, f.clone(), &mut info);

    let result = info.get(&(s, f));
    if let Some(true) = result {
        true
    } else {
        false
    }
}