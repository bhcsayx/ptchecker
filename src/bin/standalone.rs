
use ptchecker::utils::*;

fn main() {
    // let nets = pnets_read_ptnets_from("data/SatelliteMemory-PT-X00100Y0003.pnml");
    let nets = parse_pnml("data/SatelliteMemory-PT-X00100Y0003.pnml");
    println!("read nets: {:#?}", nets);
}