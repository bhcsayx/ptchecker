use ptchecker::logics::*;
use ptchecker::logics::parser::*;
use ptchecker::ltl::*;
use ptchecker::petri::*;
use ptchecker::petri::parser::*;
use ptchecker::utils::*;

use std::env;
use std::path::Path;
use std::process::exit;

fn ltl_check(input: &Formula) {
    println!("formula: {:?}", input.ty);
    // Test formula negation
    println!("negated: {:?}", ltl_negate(input.ty.clone()));
    // Test formula simplification
    println!("simplified: {:?}", ltl_simplify(ltl_negate(input.ty.clone())));
}

fn old_main() {
    // let nets = pnets_read_ptnets_from("data/SatelliteMemory-PT-X00100Y0003.pnml");
    // let nets = parse_pnml("data/SatelliteMemory-PT-X00100Y0003.pnml");
    // println!("read nets: {:#?}", nets);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: ptchecker <path>\n");
        exit(0);
    }
    if !validate_path(args[1].as_str()) {
        println!("Invalid input path\n");
        exit(1);
    }
    let model_path = Path::new(args[1].as_str()).join("model.pnml");
    let nets = parse_pnml(model_path.to_str().unwrap()).unwrap_or(Vec::new());
    if nets.len() == 0 {
        println!("No model found, exiting\n");
        exit(0);
    }
    // println!("read nets: {:#?}", nets[0]);
    let input_path = Path::new(args[1].as_str()).join("LTLFireability.xml");
    if let Ok(formulas) = parse_formulas(input_path.to_str().unwrap()) {
        for f in formulas.iter() {
            // ltl_check(f);
        }
        ltl_check(&formulas[15]);
    }
}