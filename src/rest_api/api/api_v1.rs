use crate::indexing::index_manager::IndexManager;
use crate::rest_api::api::{
    v1_convert_mol_block_to_smiles, v1_convert_smiles_to_mol_block, v1_delete_index,
    v1_delete_index_bulk, v1_get_index, v1_index_search_basic, v1_index_search_identity,
    v1_index_search_structure, v1_list_indexes, v1_list_schemas, v1_merge_segments, v1_post_index,
    v1_post_index_bulk, v1_standardize, BulkRequest, ConvertedMolBlockResponse,
    ConvertedSmilesResponse, DeleteIndexResponse, DeleteIndexesBulkDeleteResponse,
    GetIndexResponse, GetQuerySearchResponse, GetStructureSearchResponse, ListIndexesResponse,
    ListSchemasResponse, MergeSegmentsResponse, PostIndexResponse, PostIndexesBulkIndexResponse,
    StandardizeResponse,
};
use crate::rest_api::models::{MolBlock, Smiles};

use poem::web::Data;
use poem_openapi::{
    param::{Path, Query},
    payload::Json,
    OpenApi,
};

#[derive(Default)]
pub struct ApiV1 {}

#[OpenApi]
impl ApiV1 {
    #[oai(path = "/v1/standardize", method = "post")]
    /// Pass a list of SMILES through fragment_parent, uncharger, and canonicalization routines
    pub async fn v1_standardize(
        &self,
        mol: Json<Vec<Smiles>>,
        attempt_fix: Query<Option<String>>,
    ) -> StandardizeResponse {
        v1_standardize(mol, attempt_fix.0.as_deref()).await
    }

    #[oai(path = "/v1/convert/mol_block_to_smiles", method = "post")]
    /// Convert a list of SMILES to molblocks
    pub async fn v1_convert_mol_block_to_smiles(
        &self,
        sanitize: Query<String>,
        mol_blocks: Json<Vec<MolBlock>>,
    ) -> ConvertedSmilesResponse {
        v1_convert_mol_block_to_smiles(sanitize.0, mol_blocks).await
    }

    #[oai(path = "/v1/convert/smiles_to_mol_block", method = "post")]
    /// Convert a list of molblocks to SMILES
    pub async fn v1_convert_smiles_to_mol_block(
        &self,
        smiles_vec: Json<Vec<Smiles>>,
    ) -> ConvertedMolBlockResponse {
        v1_convert_smiles_to_mol_block(smiles_vec).await
    }

    #[oai(path = "/v1/schemas", method = "get")]
    /// List schemas available for creating indexes
    pub async fn v1_list_schemas(&self) -> ListSchemasResponse {
        v1_list_schemas().await
    }

    #[oai(path = "/v1/indexes", method = "get")]
    /// List indexes
    pub async fn v1_list_indexes(&self, index_manager: Data<&IndexManager>) -> ListIndexesResponse {
        v1_list_indexes(index_manager.0)
    }

    #[oai(path = "/v1/indexes/:index", method = "get")]
    /// Get extended information about an index
    pub async fn v1_get_index(
        &self,
        index: Path<String>,
        index_manager: Data<&IndexManager>,
    ) -> GetIndexResponse {
        v1_get_index(index_manager.0, index.to_string())
    }

    // v1/indexes/inventory_items_v1?schema=v1_descriptors
    #[oai(path = "/v1/indexes/:index", method = "post")]
    /// Create an index
    pub async fn v1_post_index(
        &self,
        index: Path<String>,
        schema: Query<String>,
        index_manager: Data<&IndexManager>,
    ) -> PostIndexResponse {
        v1_post_index(index_manager.0, index.to_string(), schema.0)
    }

    // v1/indexes/inventory_items_v1/merge
    #[oai(path = "/v1/indexes/:index/merge", method = "post")]
    /// Merge segments inside the index
    pub async fn v1_post_index_merge_segments(
        &self,
        index: Path<String>,
        index_manager: Data<&IndexManager>,
    ) -> MergeSegmentsResponse {
        v1_merge_segments(index_manager.0, index.to_string()).await
    }

    #[oai(path = "/v1/indexes/:index", method = "delete")]
    /// Delete an index
    pub async fn v1_delete_index(
        &self,
        index: Path<String>,
        index_manager: Data<&IndexManager>,
    ) -> DeleteIndexResponse {
        v1_delete_index(index_manager.0, index.to_string())
    }

    #[oai(path = "/v1/indexes/:index/bulk_index", method = "post")]
    /// Index a list of SMILES and associated, free-form JSON attributes
    /// which are indexed and searchable
    pub async fn v1_post_indexes_bulk_index(
        &self,
        index: Path<String>,
        bulk_request: Json<BulkRequest>,
        index_manager: Data<&IndexManager>,
    ) -> PostIndexesBulkIndexResponse {
        v1_post_index_bulk(index_manager.0, index.to_string(), bulk_request.0).await
    }

    #[oai(path = "/v1/indexes/:index/bulk_delete", method = "delete")]
    /// Delete a list of smiles (after standardization) from an index
    pub async fn v1_delete_indexes_bulk_delete(
        &self,
        index: Path<String>,
        bulk_request: Json<BulkRequest>,
        index_manager: Data<&IndexManager>,
    ) -> DeleteIndexesBulkDeleteResponse {
        v1_delete_index_bulk(index_manager.0, index.to_string(), bulk_request.0).await
    }

    #[oai(path = "/v1/indexes/:index/search/basic", method = "get")]
    /// Perform basic query search against index
    pub async fn v1_index_search_basic(
        &self,
        index: Path<String>,
        query: Query<String>,
        limit: Query<Option<usize>>,
        index_manager: Data<&IndexManager>,
    ) -> GetQuerySearchResponse {
        let limit = limit.0.unwrap_or(1000);

        v1_index_search_basic(index_manager.0, index.to_string(), query.0, limit)
    }

    #[allow(clippy::too_many_arguments)]
    #[oai(path = "/v1/indexes/:index/search/substructure", method = "get")]
    /// Perform substructure search against index
    pub async fn v1_index_search_substructure(
        &self,
        index: Path<String>,
        smiles: Query<String>,
        use_chirality: Query<Option<bool>>,
        result_limit: Query<Option<usize>>,
        tautomer_limit: Query<Option<usize>>,
        extra_query: Query<Option<String>>,
        use_scaffolds: Query<Option<bool>>,
        index_manager: Data<&IndexManager>,
    ) -> GetStructureSearchResponse {
        let use_chirality = use_chirality.0.unwrap_or(false);
        let result_limit = result_limit.0.unwrap_or(1000);
        let tautomer_limit = tautomer_limit.0.unwrap_or(0);
        let extra_query = extra_query.0.unwrap_or_default();
        let use_scaffolds = use_scaffolds.0.unwrap_or(true);

        let index = index_manager.0.open(&index);

        v1_index_search_structure(
            index,
            smiles.0,
            use_chirality,
            "substructure",
            result_limit,
            tautomer_limit,
            &extra_query,
            use_scaffolds,
        )
    }

    #[allow(clippy::too_many_arguments)]
    #[oai(path = "/v1/indexes/:index/search/superstructure", method = "get")]
    /// Perform superstructure search against index
    pub async fn v1_index_search_superstructure(
        &self,
        index: Path<String>,
        smiles: Query<String>,
        use_chirality: Query<Option<String>>,
        result_limit: Query<Option<usize>>,
        tautomer_limit: Query<Option<usize>>,
        extra_query: Query<Option<String>>,
        use_scaffolds: Query<Option<String>>,
        index_manager: Data<&IndexManager>,
    ) -> GetStructureSearchResponse {
        // by default, we will ignore chirality
        let use_chirality = if let Some(use_chirality) = use_chirality.0 {
            !matches!(use_chirality.as_str(), "false")
        } else {
            false
        };

        let result_limit = if let Some(result_limit) = result_limit.0 {
            result_limit
        } else {
            usize::try_from(1000).unwrap()
        };

        let tautomer_limit = if let Some(tautomer_limit) = tautomer_limit.0 {
            tautomer_limit
        } else {
            usize::try_from(0).unwrap()
        };

        let extra_query = if let Some(extra_query) = extra_query.0 {
            extra_query
        } else {
            "".to_string()
        };

        // by default, we will use scaffold-based indexing
        let use_scaffolds = if let Some(use_scaffolds) = use_scaffolds.0 {
            matches!(use_scaffolds.as_str(), "true")
        } else {
            true
        };

        let index = index_manager.0.open(&index);

        v1_index_search_structure(
            index,
            smiles.0,
            use_chirality,
            "superstructure",
            result_limit,
            tautomer_limit,
            &extra_query,
            use_scaffolds,
        )
    }

    #[oai(path = "/v1/indexes/:index/search/identity", method = "get")]
    /// Perform identity search (i.e. exact match) against index
    pub async fn v1_index_search_identity(
        &self,
        index: Path<String>,
        smiles: Query<String>,
        use_chirality: Query<Option<String>>,
        extra_query: Query<Option<String>>,
        use_scaffolds: Query<Option<String>>,
        index_manager: Data<&IndexManager>,
    ) -> GetStructureSearchResponse {
        // by default, we will ignore chirality
        let use_chirality = if let Some(use_chirality) = use_chirality.0 {
            !matches!(use_chirality.as_str(), "false")
        } else {
            false
        };

        let extra_query = if let Some(extra_query) = extra_query.0 {
            extra_query
        } else {
            "".to_string()
        };

        // by default, we will use scaffold-based indexing
        let use_scaffolds = if let Some(use_scaffolds) = use_scaffolds.0 {
            matches!(use_scaffolds.as_str(), "true")
        } else {
            true
        };

        v1_index_search_identity(
            index_manager.0,
            index.to_string(),
            smiles.0,
            use_chirality,
            &extra_query,
            use_scaffolds,
        )
    }
}
