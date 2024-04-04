use bitvec::macros::internal::funty::Fundamental;
use std::collections::HashMap;

use rdkit::MolSanitizeException::{AtomValenceException, KekulizeException};
use rdkit::{
    detect_chemistry_problems, fragment_parent, set_hybridization, substruct_match, Atom,
    CleanupParameters, Fingerprint, HybridizationType, Properties, ROMol, RWMol,
    SmilesParserParams, SubstructMatchParameters, TautomerEnumerator,
};

pub fn update_atom_hcount(atom: &mut Atom, chg: i32, num_h: i32) {
    atom.set_formal_charge(chg);
    atom.set_num_explicit_hs(num_h);
    atom.update_property_cache(true);
}

pub fn neutralize_atoms(romol: &ROMol) -> eyre::Result<ROMol> {
    let mut neutralized_romol = romol.clone();
    let pattern =
        RWMol::from_smarts("[+1!h0!$([*]~[-1,-2,-3,-4]),-1!$([*]~[+1,+2,+3,+4])]").unwrap();
    let params = SubstructMatchParameters::new();
    let atom_matches = substruct_match(romol, &pattern.clone().to_ro_mol(), &params);

    let atom_match_pos = atom_matches
        .into_iter()
        .map(|v| v[0].mol_atom_idx.as_u32())
        .collect::<Vec<_>>();

    if atom_match_pos.len() > 0 {
        for atom_idx in atom_match_pos {
            let mut atom = neutralized_romol.atom_with_idx(atom_idx);

            // Do not modify charged aromatic atoms
            if atom.get_is_aromatic() {
                continue;
            }

            let hybridization_type = atom.get_hybridization_type();
            let chg = atom.get_formal_charge();
            let hcount = atom.get_total_num_hs() as i32;

            if hybridization_type == HybridizationType::SP3 && chg < 0 {
                continue;
            }

            update_atom_hcount(&mut atom, 0, hcount - chg);
            set_hybridization(&mut neutralized_romol);
        }
    }

    Ok(neutralized_romol)
}

pub fn remove_hypervalent_silicon(smi: &str) -> String {
    let hyperval_si = "[Si-";
    if smi.contains(hyperval_si) && smi.contains(".") {
        smi.split(".")
            .map(|f| match f.contains(hyperval_si) {
                true => "",
                false => f,
            })
            .collect::<Vec<_>>()
            .join("")
    } else {
        smi.to_string()
    }
}

pub fn add_formal_charge(romol: &mut ROMol, atom_idx: u32) {
    romol.update_property_cache(false);

    let expected_atoms = ["C", "N", "O"];
    let valencies = [4, 3, 2];
    let valence_hash = expected_atoms
        .into_iter()
        .zip(valencies)
        .map(|(k, v)| (k.to_string(), v))
        .collect::<HashMap<String, u32>>();

    let mut atom = romol.atom_with_idx(atom_idx);
    let atom_symbol = &atom.symbol()[..];

    if expected_atoms.contains(&atom_symbol) {
        let total_val = atom.get_total_valence() as i32;
        let formal_chg = atom.get_formal_charge();
        let expected_val = valence_hash[atom_symbol] as i32;

        if total_val - formal_chg != expected_val {
            let new_formal_chg = total_val - expected_val;
            atom.set_formal_charge(new_formal_chg);
        }
    }
}

pub fn fix_chemistry_problems(smi: &str) -> eyre::Result<ROMol> {
    let mut fixed_smi = smi.to_string();
    let mut parser_params = SmilesParserParams::default();
    parser_params.set_sanitize(false);

    let mut romol = ROMol::from_smiles_with_params(fixed_smi.as_str(), &parser_params)
        .map_err(|e| eyre::eyre!("{}", e))?;
    let mut problems = detect_chemistry_problems(&romol);

    // Fix smiles AND romol for each problem
    for problem in problems {
        match problem {
            AtomValenceException { atom_idx } => {
                let atom_symbol = &romol.atom_with_idx(atom_idx).symbol()[..];
                if atom_symbol == "Si" {
                    let new_smi = remove_hypervalent_silicon(fixed_smi.as_str());
                    fixed_smi = new_smi.clone();
                    romol = ROMol::from_smiles_with_params(fixed_smi.as_str(), &parser_params)
                        .map_err(|e| eyre::eyre!("{}", e))?;
                } else if ["C", "N", "O"].contains(&atom_symbol) {
                    add_formal_charge(&mut romol, atom_idx);
                    fixed_smi = romol.as_smiles();
                }
            }
            KekulizeException => {
                if fixed_smi.contains(&"[c-]") {
                    fixed_smi = fixed_smi.replace("[c-]", "[cH-]");
                    romol = ROMol::from_smiles_with_params(fixed_smi.as_str(), &parser_params)
                        .map_err(|e| eyre::eyre!("{}", e))?;
                }
            }
            _ => {}
        }
    }

    problems = detect_chemistry_problems(&romol);

    if problems.len() == 0 {
        // Rebuild romol to force sanitization
        Ok(ROMol::from_smiles(fixed_smi.as_str())?)
    } else {
        Err(eyre::eyre!(
            "Problems still exist after attempted molecule fix: {:?}",
            problems
        ))
    }
}

pub fn standardize_mol(romol: &ROMol) -> eyre::Result<ROMol> {
    let rwmol = romol.as_rw_mol(false, 1);
    let cleanup_params = CleanupParameters::default();
    let parent_rwmol = fragment_parent(&rwmol, &cleanup_params, true);
    let te = TautomerEnumerator::new();
    let canon_taut = te.canonicalize(&parent_rwmol.to_ro_mol());
    let neutralized_canon = neutralize_atoms(&canon_taut)?;
    Ok(neutralized_canon)
}

pub fn standardize_smiles(smi: &str, attempt_fix: bool) -> eyre::Result<ROMol> {
    let romol = match attempt_fix {
        true => fix_chemistry_problems(smi)?,
        false => ROMol::from_smiles(smi)?,
    };

    let canon_taut = standardize_mol(&romol)?;
    Ok(canon_taut)
}

pub fn get_tautomers(romol: &ROMol) -> Vec<ROMol> {
    let te = TautomerEnumerator::new();
    let ter = te.enumerate(romol);
    ter.collect::<Vec<_>>()
}

pub fn get_cpd_properties(romol: &ROMol) -> eyre::Result<(Fingerprint, HashMap<String, f64>)> {
    let properties = Properties::new();
    let computed = properties.compute_properties(romol);
    let rdkit_fp = romol.fingerprint();

    Ok((rdkit_fp, computed))
}

pub fn process_cpd(
    smi: &str,
    attempt_fix: bool,
) -> eyre::Result<(ROMol, Fingerprint, HashMap<String, f64>)> {
    let canon_taut = standardize_smiles(smi, attempt_fix)?;
    let (rdkit_fp, computed) = get_cpd_properties(&canon_taut)?;

    Ok((canon_taut, rdkit_fp, computed))
}

lazy_static::lazy_static! {
    static ref BRACKETS_RE: regex::Regex = regex::Regex::new(r"\[(?P<e>B|C|N|O|P|S|F|Cl|Br|I)\]").unwrap();
}

pub fn remove_organic_brackets(smi: &str) -> String {
    let new_smi = BRACKETS_RE.replace_all(smi, "$e");
    new_smi.as_ref().to_string()
}
