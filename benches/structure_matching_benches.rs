#![feature(test)]

extern crate test;
use cheminee::search::structure_matching::exact_match;
use rdkit::{substruct_match, ROMol, SubstructMatchParameters};
use test::Bencher;

#[bench]
fn bench_exact_match_no_chirality(b: &mut Bencher) {
    let romol1 = ROMol::from_smiles(
        "C[S+](CC[C@@H](C(=O)[O-])[NH3+])C[C@@H]1[C@H]([C@H]([C@@H](O1)N2C=NC3=C(N=CN=C32)N)O)O",
    )
    .unwrap();

    let romol2 = romol1.clone();

    b.iter(|| {
        let _ = exact_match(&romol1, &romol2, false);
    });
}

// running 1 test
// test bench_exact_match_no_chirality ... bench:      10,117.06 ns/iter (+/- 423.84)

#[bench]
fn bench_exact_match_yes_chirality(b: &mut Bencher) {
    let romol1 = ROMol::from_smiles(
        "C[S+](CC[C@@H](C(=O)[O-])[NH3+])C[C@@H]1[C@H]([C@H]([C@@H](O1)N2C=NC3=C(N=CN=C32)N)O)O",
    )
    .unwrap();

    let romol2 = romol1.clone();

    b.iter(|| {
        let _ = exact_match(&romol1, &romol2, true);
    });
}

// running 1 test
// test bench_exact_match_yes_chirality ... bench:      18,131.02 ns/iter (+/- 790.45)

#[bench]
fn bench_substructure_match_no_chirality(b: &mut Bencher) {
    let super_mol = ROMol::from_smiles(
        "C[S+](CC[C@@H](C(=O)[O-])[NH3+])C[C@@H]1[C@H]([C@H]([C@@H](O1)N2C=NC3=C(N=CN=C32)N)O)O",
    )
    .unwrap();
    let sub_mol =
        ROMol::from_smiles("C[C@@H]1[C@H]([C@H]([C@@H](O1)[N]2C=NC3=C(N=CN=C23)N)O)O").unwrap();

    let mut params = SubstructMatchParameters::default();
    params.set_use_chirality(false);

    b.iter(|| {
        let _ = substruct_match(&super_mol, &sub_mol, &params);
    });
}

// running 1 test
// test bench_substructure_match_no_chirality ... bench:       4,209.80 ns/iter (+/- 318.14)

#[bench]
fn bench_substructure_match_yes_chirality(b: &mut Bencher) {
    let super_mol = ROMol::from_smiles(
        "C[S+](CC[C@@H](C(=O)[O-])[NH3+])C[C@@H]1[C@H]([C@H]([C@@H](O1)N2C=NC3=C(N=CN=C32)N)O)O",
    )
    .unwrap();
    let sub_mol =
        ROMol::from_smiles("C[C@@H]1[C@H]([C@H]([C@@H](O1)[N]2C=NC3=C(N=CN=C23)N)O)O").unwrap();

    let mut params = SubstructMatchParameters::default();
    params.set_use_chirality(true);

    b.iter(|| {
        let _ = substruct_match(&super_mol, &sub_mol, &params);
    });
}

// running 1 test
// test bench_substructure_match_yes_chirality ... bench:       7,313.00 ns/iter (+/- 532.24)
