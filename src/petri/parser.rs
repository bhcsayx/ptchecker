use crate::petri::*;
use std::error::Error;
use std::fs;
use roxmltree::Document;

pub fn is_type_element(node: &roxmltree::Node, ty: &str) -> bool {
    node.is_element() && node.tag_name().name() == ty
}

pub fn parse_pnml(path: &str) -> Result<Vec<PTNet>, Box<dyn Error>> {
    let raw_string = fs::read_to_string(path)?;
    let mut nets = Vec::new();

    let doc = Document::parse(raw_string.as_str())?;
    let ns = doc.root_element().tag_name().namespace().unwrap_or("");

    let root_child = doc.root().first_element_child();
    if root_child.is_none() {
        return Ok(Vec::new());
    }
    // println!("first root child: {:?}", root_child);
    for child in root_child.unwrap().children().filter(|n| is_type_element(n, "net")) {
        // println!("net: {:?}\n", child);
        let net = parse_net(&child);
        nets.push(net);
    }

    Ok(nets)
}

fn parse_net(node: &roxmltree::Node) -> PTNet {
    let mut net = PTNet::default();
    // Get net name
    // println!("first child: {:?}", node.first_element_child());
    net.name = match node.first_element_child() {
        None => String::new(),
        Some(child) => match child.first_element_child() {
            None => String::new(),
            Some(grand_child) => grand_child.text().unwrap_or("").to_string(),
        },
    };
    
    // Parse pages
    for page in node.children().filter(|n| is_type_element(n, "page")) {
        parse_page(&mut net, &page);
    }

    net
}

fn parse_page(net: &mut PTNet, node: &roxmltree::Node) {
    // Get page name
    let auto_page_name = format!("auto-page-{}", net.pages.len());
    let page_name = node.attribute("id").unwrap_or(auto_page_name.as_str());
    net.pages.push(page_name.to_string());

    // Parse places
    for place in node.children().filter(|n| is_type_element(n, "place")) {
        parse_place(net, &place, page_name);
    }

    // Parse transitions
    for trans in node.children().filter(|n| is_type_element(n, "transition")) {
        parse_transitions(net, &trans, page_name);
    }

    // Parse arcs
    for arc in node.children().filter(|n| is_type_element(n, "arc")) {
        parse_arc(net, &arc);
    }
}

fn parse_place(net: &mut PTNet, node: &roxmltree::Node, page: &str) {
    let mut place = Place {
        name: String::new(),
        page: String::new(),
        init: Marking::Plain(0),
        producers: Vec::new(),
        consumers: Vec::new(),
    };
    // Get place name
    let auto_place_name = format!("auto-place-{}", net.places.len());
    place.name = match node.first_element_child() {
        None => auto_place_name,
        Some(child) => match child.first_element_child() {
            None => auto_place_name,
            Some(grand_child) => grand_child.text().unwrap_or("").to_string(),
        },
    };

    place.page = page.to_string();
    
    // Get place marking
    for marking in node.children().filter(|n| is_type_element(n, "initialMarking")) {
        match marking.first_element_child() {
            None => {},
            Some(child) => match child.text() {
                Some(txt) => {
                    place.init = Marking::Plain(txt.parse::<usize>().unwrap_or(0));
                },
                None => {},
            }
        }
    }
    net.insert_place(place);
}

fn parse_transitions(net: &mut PTNet, node: &roxmltree::Node, page: &str) {
    let mut transition = Transition {
        name: String::new(),
        page: String::new(),
        conditions: Vec::new(),
        effects: Vec::new(),
    };
    // Get transition name
    let auto_trans_name = format!("auto-trans-{}", net.transitions.len());
    transition.name = match node.first_element_child() {
        None => auto_trans_name,
        Some(child) => match child.first_element_child() {
            None => auto_trans_name,
            Some(grand_child) => grand_child.text().unwrap_or("").to_string(),
        },
    };
    transition.page = page.to_string();
    net.insert_transition(transition);
}

fn parse_arc(net: &mut PTNet, node: &roxmltree::Node) {
    // Get arc name
    let arc_id = node.attribute("id").unwrap();
    let arc_ty = match node.first_element_child() {
        None => ArcTy::Plain(1),
        Some(insc) => {
            if (insc.tag_name().name() == "inscription") { // Normal weight
                if let Some(txt) = insc.first_element_child() {
                    ArcTy::Plain(txt.text().unwrap_or("").parse::<usize>().unwrap_or(1))
                }
                else {
                    ArcTy::Plain(1)
                }
            }
            // TODO: Handle colored case in <hlinscription>
            else {
                ArcTy::Plain(1)
            }
        }
    };
    let arc_src = node.attribute("source").unwrap().to_string();
    let arc_dst = node.attribute("target").unwrap().to_string();
    let arc = Arc {
        id: arc_id.to_string(),
        ty: arc_ty,
    };
    net.insert_arc(arc, arc_src, arc_dst);
}