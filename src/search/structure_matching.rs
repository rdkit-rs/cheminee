use bitvec::prelude::BitSlice;
use rdkit::*;

pub fn exact_match(romol1: &ROMol, romol2: &ROMol, use_chirality: bool) -> bool {
    let mut params = SubstructMatchParameters::default();
    params.set_use_chirality(use_chirality);
    let match1 = substruct_match(romol1, romol2, &params);
    let match2 = substruct_match(romol2, romol1, &params);
    !(match1.is_empty() || match2.is_empty())
}

pub fn exact_match_fp(romol1: &ROMol, romol2: &ROMol) -> bool {
    let fp1 = romol1.fingerprint().0;
    let fp2 = romol2.fingerprint().0;
    fp1 == fp2
}

pub fn substructure_match_fp(
    substructure_fp: &BitSlice<u8>,
    superstructure_fp: &BitSlice<u8>,
) -> bool {
    let and_match = substructure_fp.to_bitvec() & superstructure_fp;
    and_match == substructure_fp
}
