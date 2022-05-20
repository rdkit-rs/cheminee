use rdkit::*;

pub fn mol_stdz(romol: &ROMol) -> ROMol {
    let rwmol = romol.to_rw_mol(false, 1);
    let cleanup_params = CleanupParameters::default();
    let parent_rwmol = fragment_parent(&rwmol, &cleanup_params, true);

    let uncharger = Uncharger::new(false);
    let uncharged_mol = uncharger.uncharge(&parent_rwmol.to_romol());

    let te = TautomerEnumerator::new();
    let canon_taut = te.canonicalize(&uncharged_mol);
    canon_taut
}

pub fn smi_stdz(smi: &str) -> ROMol {
    let romol = ROMol::from_smile(smi).unwrap();
    let canon_taut = mol_stdz(&romol);
    canon_taut
}
