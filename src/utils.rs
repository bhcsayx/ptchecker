// use pnets::standard;
// use pnets_pnml::pnml::Pnml;
// use serde_xml_rs::from_str;
// use quick_xml::Reader;
// use quick_xml::events::Event;
// use quick_xml::name::QName;
use std::convert::TryInto;
use std::path::Path;

// pub fn pnets_read_ptnets_from(path: &str) -> Result<Vec<standard::Net>, Box<dyn Error>> {
//     let raw_string = fs::read_to_string(path)?;
//     let pnml: Pnml = quick_xml::de::from_str(raw_string.as_str())?;
//     let nets: Vec<standard::Net> = (&pnml).try_into()?;
//     // let pnml: Pnml = (&nets).into();
//     for net in nets.iter() {
//         println!("net: {:?}", net);
//     }
//     // println!("{:?}", quick_xml::se::to_string(&pnml));
//     Ok(nets)
// }

pub fn validate_path(input: &str) -> bool {
    let path = Path::new(input);
    if !path.exists() {
        return false;
    }
    let model_path = path.join("model.pnml");
    if !model_path.exists() {
        return false;
    }
    let ltl_fire_path = path.join("LTLFireability.txt");
    if !ltl_fire_path.exists() {
        return false;
    }
    let ltl_card_path = path.join("LTLCardinality.xml");
    if !ltl_card_path.exists() {
        return false;
    }
    let ctl_fire_path = path.join("CTLFireability.txt");
    if !ctl_fire_path.exists() {
        return false;
    }
    let ctl_card_path = path.join("CTLCardinality.xml");
    if !ctl_card_path.exists() {
        return false;
    }
    return true;
}