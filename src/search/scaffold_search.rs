use rdkit::{substruct_match, ROMol, SubstructMatchParameters};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::Field;
use tantivy::{DocAddress, Index, Searcher};

pub fn get_scaffold_docs(index: &Index, searcher: &Searcher) -> eyre::Result<Vec<DocAddress>> {
    let query_parser = QueryParser::for_index(index, vec![]);
    let query = query_parser.parse_query("*")?;
    let arbitrary_collector = &TopDocs::with_limit(10_000);
    let all_scaffolds = searcher
        .search(&query, arbitrary_collector)?
        .into_iter()
        .map(|result| result.1)
        .collect::<Vec<DocAddress>>();

    Ok(all_scaffolds)
}

pub fn scaffold_search(
    query_mol: &ROMol,
    scaffold_docs: &Vec<DocAddress>,
    smiles_field: Field,
    id_field: Field,
    searcher: &Searcher,
) -> eyre::Result<Vec<u64>> {
    let mut matching_scaffolds: Vec<u64> = Vec::with_capacity(scaffold_docs.len());
    for docaddr in scaffold_docs {
        let doc = searcher.doc(*docaddr)?;

        let scaffold_smiles = doc
            .get_first(smiles_field)
            .ok_or(eyre::eyre!("Tantivy smiles retrieval failed"))?
            .as_text()
            .ok_or(eyre::eyre!("Failed to stringify smiles"))?;
        let scaffold_id = doc
            .get_first(id_field)
            .ok_or(eyre::eyre!("Tantivy id retrieval failed"))?
            .as_u64()
            .ok_or(eyre::eyre!("Failed to convert id to u64"))?;

        let params = SubstructMatchParameters::default();
        let mol_substruct_match =
            substruct_match(&query_mol, &ROMol::from_smiles(scaffold_smiles)?, &params);
        if !mol_substruct_match.is_empty() {
            matching_scaffolds.push(scaffold_id);
        }
    }

    Ok(matching_scaffolds)
}
