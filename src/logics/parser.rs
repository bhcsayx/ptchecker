use crate::logics::*;
use std::error::Error;
use std::fs;
use std::process::exit;
use roxmltree::Document;

pub fn parse_formulas(path: &str) -> Result<Vec<Formula>, Box<dyn Error>> {
    let raw_string = fs::read_to_string(path)?;
    let mut formulas = Vec::new();

    let doc = Document::parse(raw_string.as_str())?;
    // let mut i = 0;
    for p in doc.root_element().children() {
        if p.is_element() {
            // Property node
            // println!("parsing formula {}", i);
            let mut formula = Formula {
                name: String::new(),
                ty: FormulaTy::False,
            };
            for c in p.children() {
                if c.is_element() {
                    match c.tag_name().name() {
                        "id" => formula.name = String::from(c.text().unwrap_or("")),
                        "formula" => {
                            formula.ty = parse_formula(&c.first_element_child().unwrap());
                        },
                        _ => {}
                    }
                }
            }
            formulas.push(formula);
        }
    }

    Ok(formulas)
}

fn parse_formula(node: &roxmltree::Node) -> FormulaTy {
    // let formula_node = node.first_element_child().unwrap();
    let formula_node = node.clone();
    match formula_node.tag_name().name() {
        "is-fireable" => {
            let transition = formula_node.first_element_child().unwrap();
            return FormulaTy::Prop(
                PTAtom::Fireability(String::from(transition.text().unwrap()))
            );
        },
        "integer-le" => {
            let elements: Vec<roxmltree::Node> = formula_node.children().filter(|c| c.is_element()).collect();
            if elements.len() != 2 {
                println!("Malformed cardinality atoms");
                exit(1);
            }
            let lhs = match elements[0].tag_name().name() {
                "integer-constant" => format!("n_{}", elements[0].text().unwrap()),
                "tokens-count" => format!("p_{}", elements[0].text().unwrap()),
                _ => {
                    println!("Malformed cardinality operators");
                    exit(1);
                }
            };
            let rhs = match elements[1].tag_name().name() {
                "integer-constant" => format!("n_{}", elements[1].text().unwrap()),
                "tokens-count" => format!("p_{}", elements[1].text().unwrap()),
                _ => {
                    println!("Malformed cardinality operators");
                    exit(1);
                }
            };
            return FormulaTy::Prop(
                PTAtom::Cardinality(lhs, rhs)
            );
        },
        "negation" => {
            let inner = formula_node.first_element_child().unwrap();
            // println!("n: {:?}", formula_node);
            return FormulaTy::Not(
                Box::new(parse_formula(&inner))
            );
        },
        "conjunction" => {
            let elements: Vec<roxmltree::Node> = formula_node.children().filter(|c| c.is_element()).collect();
            let mut ret = FormulaTy::And(
                Box::new(parse_formula(&elements[0])),
                Box::new(parse_formula(&elements[1]))
            );
            for i in 2..elements.len() {
                ret = FormulaTy::And(
                    Box::new(ret.clone()),
                    Box::new(parse_formula(&elements[i]))
                );
            }
            return ret;
        },
        "disjunction" => {
            let elements: Vec<roxmltree::Node> = formula_node.children().filter(|c| c.is_element()).collect();
            let mut ret = FormulaTy::Or(
                Box::new(parse_formula(&elements[0])),
                Box::new(parse_formula(&elements[1]))
            );
            for i in 2..elements.len() {
                ret = FormulaTy::Or(
                    Box::new(ret.clone()),
                    Box::new(parse_formula(&elements[i]))
                );
            }
            return ret;
        },
        "finally" => {
            let inner = formula_node.first_element_child().unwrap();
            return FormulaTy::Finally(
                Box::new(parse_formula(&inner))
            );
        },
        "globally" => {
            let inner = formula_node.first_element_child().unwrap();
            return FormulaTy::Global(
                Box::new(parse_formula(&inner))
            );
        },
        "next" => {
            let inner = formula_node.first_element_child().unwrap();
            // println!("inner n: {:?}", inner);
            return FormulaTy::Next(
                Box::new(parse_formula(&inner))
            );
        },
        "until" => {
            let elements: Vec<roxmltree::Node> = formula_node.children().filter(|c| c.is_element()).collect();
            if elements.len() != 2 || elements[0].tag_name().name() != "before" || elements[1].tag_name().name() != "reach" {
                println!("Malformed until");
                exit(1);
            }
            return FormulaTy::Until(
                Box::new(parse_formula(&elements[0].first_element_child().unwrap())),
                Box::new(parse_formula(&elements[1].first_element_child().unwrap()))
            );
        },
        "all-paths" => {
            let inner = formula_node.first_element_child().unwrap();
            return FormulaTy::Forall(
                Box::new(parse_formula(&inner))
            );
        },
        "exists-path" => {
            let inner = formula_node.first_element_child().unwrap();
            return FormulaTy::Exists(
                Box::new(parse_formula(&inner))
            );
        }
        _ => {
            println!("Malformed formula: {:?}", formula_node);
            exit(1);
        }
    }
}