use rdkit::*;

pub fn exact_match(romol1: &ROMol, romol2: &ROMol) -> bool {
    let match1 = substruct_match(romol1, romol2);
    let match2 = substruct_match(romol2, romol1);
    match1 & match2
}

pub fn exact_match_fp(romol1: &ROMol, romol2: &ROMol) -> bool {
    let fp1 = romol1.fingerprint().0;
    let fp1_clone = fp1.clone();
    let fp2 = romol2.fingerprint().0;
    (fp1 & fp2) == fp1_clone
}
