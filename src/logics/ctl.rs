use std::collections::{HashSet, HashMap};
use crate::logics::{FormulaTy, PTAtom};
use crate::logics::transys::{State, TranSys};


pub fn visit_eu(T: &TranSys, s: State, f: FormulaTy, info: &mut HashMap<(State, String), bool>, marks: & mut HashSet<State>) {
    marks.insert(s);
    let fc = f.clone();
    if info.contains_key(&(s, f.to_string())) {
        if let Some(true) = info.get(&(s, f.to_string())) {
            info.insert((s, f.to_string()), true);
        }
    } else if let FormulaTy::Until(f1, f2) = fc {
        check(T, s, *f2.clone(), info);
        if let Some(true) = info.get(&(s, f2.to_string())) {
            info.insert((s, f.to_string()), true);
        }

        check(T, s,*f1.clone(), info);
        if let Some(false) = info.get(&(s, f1.to_string())) {
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

pub fn checkEU(T: &TranSys, sr: State, f: FormulaTy, info: &mut HashMap<(State, String), bool>, marks: & mut HashSet<State>) {
    visit_eu(T, sr, f, info, marks);
}

pub fn visit_au(T: &TranSys, s: State, f: FormulaTy, info: &mut HashMap<(State, String), bool>, cp: &mut Vec<State>) {
    let fc = f.clone();
    if info.contains_key(&(s, f.to_string())) {
        if let Some(false) = info.get(&(s, f.to_string())) {
            for s1 in cp.iter() {
                info.insert((*s1, f.to_string()), false);
            }
        }
    } else if let FormulaTy::Forall(boxed_formula) = fc {
        if let FormulaTy::Until(f1, f2) = *boxed_formula {
            check(T, s,*f2.clone(), info);
            if let Some(true) = info.get(&(s, f2.to_string())) {
                info.insert((s, f.to_string()), true);
                return;
            }

            check(T, s,*f1.clone(), info);
            if let Some(false) = info.get(&(s,f1.to_string())) {
                info.insert((s, f.to_string()), false);
                for s1 in cp.iter() {
                    info.insert((*s1, f.to_string()), false);
                }
                return;
            }

            cp.push(s);
            if let Some(&ref set) = T.states_from(&s) {
                for s1 in set {
                    if !cp.contains(&s1) {
                        visit_au(T, *s1, f.clone(), info, cp);
                    } else {
                        for s1 in cp.iter() {
                            info.insert((*s1, f.to_string()), false);
                        }
                    }
                }
            }
            info.insert((s, f.to_string()), true);
            cp.pop();
        }
    }
}


pub fn checkAU(T: &TranSys, sr: State, f: FormulaTy, info: &mut HashMap<(State, String), bool>) {
    let mut cp = Vec::new();
    visit_au(T, sr, f, info, &mut cp);
}

pub fn simplify(f: FormulaTy) -> FormulaTy {
    match f {
        FormulaTy::Not(f) => {
            return FormulaTy::Not(Box::new(simplify(*f)));
        }
        FormulaTy::Or(f1, f2) => {
            return FormulaTy::Or(Box::new(simplify(*f1)), Box::new(simplify(*f2)));
        }
        FormulaTy::And(f1, f2) => {
            return FormulaTy::And(Box::new(simplify(*f1)), Box::new(simplify(*f2)));
        }
        FormulaTy::Next(f) => {
            return FormulaTy::Next(Box::new(simplify(*f)));
        }
        FormulaTy::Global(f1) => {
            return FormulaTy::Until(Box::new(FormulaTy::False), f1);
        }
        FormulaTy::Finally(f1) => {
            return FormulaTy::Until(Box::new(FormulaTy::True), f1);
        }
        FormulaTy::Until(f1, f2) => {
            return FormulaTy::Until(Box::new(simplify(*f1)), Box::new(simplify(*f2)));
        }
        FormulaTy::Forall(f) => {
            return FormulaTy::Forall(Box::new(simplify(*f)));
        }
        FormulaTy::Exists(f) => {
            return FormulaTy::Exists(Box::new(simplify(*f)));
        }
        _ => return f
    }
}

pub fn check(T: &TranSys, s: State, f: FormulaTy, info: &mut HashMap<(State, String), bool>) {
    let fc = f.clone();
    if let None = info.get(&(s, f.to_string())) {
        match fc {
            FormulaTy:: True => {
                info.insert((s, f.to_string()), true);
            }
            FormulaTy:: False => {
                info.insert((s, f.to_string()), false);
            }
            FormulaTy::Prop(f1) => {
                let name = match f1 {
                    PTAtom::Fireability(name) => {name}
                    _ => { panic!("Cardinality not supported") }
                };
                if let Some(&ref set) = T.label_of(&name) {
                    info.insert((s, f.to_string()),
                                if set.contains(&s) { true } else { false }
                    );
                }
            }
            FormulaTy::Not(f1) => {
                check(T, s, *f1.clone(), info);
                if let Some(&b) = info.get(&(s,f1.to_string())) {
                    info.insert((s, f.to_string()), !b);
                }
            }
            FormulaTy::Or(f1, f2) => {
                check(T, s, *f1.clone(), info);
                if let Some(&b) = info.get(&(s,f1.to_string())) {
                    if b {
                        info.insert((s, f.to_string()), true);
                    } else {
                        check(T, s, *f2.clone(), info);
                        if let Some(&b) = info.get(&(s, f2.to_string())) {
                            info.insert((s, f.to_string()), b);
                        }
                    }
                }
            }
            FormulaTy::And(f1, f2) => {
                check(T, s, *f1.clone(), info);
                if let Some(&b) = info.get(&(s,f1.to_string())) {
                    if b {
                        check(T, s, *f2.clone(), info);
                        if let Some(&b) = info.get(&(s, f2.to_string())) {
                            info.insert((s, f.to_string()), b);
                        }
                    } else {
                        info.insert((s, f.to_string()), false);
                    }
                }
            }
            FormulaTy::Next(f1) => {
                if let Some(&ref set) = T.states_from(&s) {
                    for s1 in set {
                        check(T, *s1, *f1.clone(), info);
                        if let Some(&b) = info.get(&(*s1, f1.to_string())) {
                            info.insert((s, f.to_string()), true);
                            return;
                        }
                    }
                }
                info.insert((s, f.to_string()), false);
            }

            FormulaTy::Forall(f1) => {
                match *f1 {
                    FormulaTy::Next(f2) => {
                        if let Some(&ref set) = T.states_from(&s) {
                            for s1 in set {
                                check(T, *s1,*f2.clone(), info);
                                if let Some(false) = info.get(&(*s1, f2.to_string())) {
                                    info.insert((s, f.to_string()), false);
                                    return;
                                }
                            }
                            info.insert((s, f.to_string()), true);
                        }
                    }
                    FormulaTy::Until(_, _) => {
                        checkAU(T, s, f.clone(), info);
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
                                if let Some(true) = info.get(&(*s1, f2.to_string())) {
                                    info.insert((s, f.to_string()), true);
                                    return;
                                }
                            }
                        }
                        info.insert((s, f.to_string()), false);
                    }
                    FormulaTy::Until(_, _) => {
                        let mut marks = HashSet::new();
                        checkEU(T, s, f.clone(), info, &mut marks);
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
    let mut info: HashMap<(State, String), bool> = HashMap::new();
    let f_simpl = simplify(f);
    println!("{}", f_simpl.to_string());
    check(&T, s, f_simpl.clone(), &mut info);
    let result = info.get(&(s, f_simpl.to_string()));
    *result.unwrap()
}