use crate::rest_api::api::{
    ScaffoldNetworkRequest, ScaffoldNetworkResponse, ScaffoldNetworkResponseError,
};
use crate::search::scaffold_network::{scaffold_networks_for_smiles, ScaffoldNetworkOptions};
use poem_openapi::payload::Json;

pub async fn v1_scaffold_network(request: ScaffoldNetworkRequest) -> ScaffoldNetworkResponse {
    let options = request
        .params
        .map(|p| ScaffoldNetworkOptions {
            include_generic_scaffolds: p.include_generic_scaffolds,
            include_generic_bond_scaffolds: p.include_generic_bond_scaffolds,
            include_scaffolds_with_attachments: p.include_scaffolds_with_attachments,
            include_scaffolds_without_attachments: p.include_scaffolds_without_attachments,
            keep_only_first_fragment: p.keep_only_first_fragment,
            prune_before_fragmenting: p.prune_before_fragmenting,
            flatten_isotopes: p.flatten_isotopes,
            flatten_chirality: p.flatten_chirality,
            flatten_keep_largest: p.flatten_keep_largest,
            collect_mol_counts: p.collect_mol_counts,
            bond_breaker_smarts: p.bond_breaker_smarts,
            standardize: p.standardize,
            max_atoms: p.max_atoms,
            max_nodes: p.max_nodes,
        })
        .unwrap_or_default();

    let smiles_vec = request
        .smiles
        .into_iter()
        .map(|s| s.smiles)
        .collect::<Vec<_>>();

    // fragmenting is CPU bound work in RDKit that cannot be interrupted, so it goes on the
    // blocking pool rather than tying up an async worker for the length of the batch
    let networks =
        tokio::task::spawn_blocking(move || scaffold_networks_for_smiles(&smiles_vec, &options))
            .await;

    match networks {
        Ok(networks) => ScaffoldNetworkResponse::Ok(Json(networks)),
        // per-molecule failures are already reported inside the 200, so getting here means
        // the blocking task itself died and there is nothing partial worth returning
        Err(e) => ScaffoldNetworkResponse::Err(Json(ScaffoldNetworkResponseError {
            error: e.to_string(),
        })),
    }
}
