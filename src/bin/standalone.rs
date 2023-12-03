use ptchecker::logics::*;
use ptchecker::logics::parser::*;
use ptchecker::petri::*;
use ptchecker::petri::parser::*;
use ptchecker::utils::*;

use std::env;
use std::path::Path;
use std::process::exit;
use ptchecker::logics::ctl::almc;

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
    println!("read nets: {:#?}", nets[0]);
    let input_path = Path::new(args[1].as_str()).join("CTLFireability.xml");
    if let Ok(formulas) = parse_formulas(input_path.to_str().unwrap()) {
        // for f in formulas {
        //     println!("formula: {:?}", f);
        // }
        println!("{}", test(&nets[0], &formulas[15]))
    }
}

fn test(model: &PTNet, formula: &Formula) -> bool {
    use ptchecker::logics::transys::*;
    let tran = TranSys::from_petri(&model);
    almc(tran, 0, formula.ty.clone())
}