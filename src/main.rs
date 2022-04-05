use rdkit_sys::molecule::Molecule;

fn main() {
    let mol = Molecule::new("CCCN", "").unwrap();
    println!("{:#?}", mol.get_descriptors_as_dict());
}
