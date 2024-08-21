use crate::indexing::index_manager::IndexManager;
use crate::rest_api::api::{
    v1_convert_mol_block_to_smiles, v1_convert_smiles_to_mol_block, v1_delete_index,
    v1_delete_index_bulk, v1_get_index, v1_index_search_basic, v1_index_search_identity,
    v1_index_search_similarity, v1_index_search_structure, v1_list_indexes, v1_list_schemas,
    v1_post_index, v1_post_index_bulk, v1_standardize, BulkRequest, ConvertedMolBlockResponse,
    ConvertedSmilesResponse, DeleteIndexResponse, DeleteIndexesBulkDeleteResponse,
    GetIndexResponse, GetQuerySearchResponse, GetStructureSearchResponse, ListIndexesResponse,
    ListSchemasResponse, PostIndexResponse, PostIndexesBulkIndexResponse, StandardizeResponse,
};
use crate::rest_api::models::{MolBlock, Smiles};

use poem::{listener::TcpListener, EndpointExt, Route, Server};
use poem_openapi::{
    param::{Path, Query},
    payload::Json,
    ContactObject, OpenApi, OpenApiService,
};
use std::path::PathBuf;

const API_PREFIX: &str = "/api";

pub fn api_service(
    server_url: &str,
    api_prefix: &str,
    indexes_root: PathBuf,
    create_storage_dir_if_missing: bool,
) -> eyre::Result<OpenApiService<Api, ()>> {
    let api = Api {
        index_manager: IndexManager::new(indexes_root, create_storage_dir_if_missing)?,
    };
    let openapi_service = OpenApiService::new(api, "Cheminée", "1.0")
        .server(format!("{}{}", server_url, api_prefix))
        .description("Cheminée: The Chemical Structure Search Engine")
        .contact(ContactObject::new().url("https://github.com/rdkit-rs/cheminee"));
    Ok(openapi_service)
}

pub async fn run_api_service(
    bind: &str,
    server_url: &str,
    index_path: PathBuf,
    create_storage_dir_if_missing: bool,
) -> eyre::Result<()> {
    let api_service = api_service(
        server_url,
        API_PREFIX,
        index_path,
        create_storage_dir_if_missing,
    )?;
    let ui = api_service.swagger_ui();

    let spec = api_service.spec();

    let logging_middleware = poem::middleware::Tracing;

    let app = Route::new()
        .at(
            "/api/v1/openapi.json",
            poem::endpoint::make_sync(move |_| spec.clone()),
        )
        .nest(API_PREFIX, api_service)
        .nest("/", ui)
        .with(logging_middleware);
    Server::new(TcpListener::bind(bind))
        .run_with_graceful_shutdown(
            app,
            async move {
                let _ = tokio::signal::ctrl_c().await;
            },
            Some(tokio::time::Duration::from_secs(5)),
        )
        .await?;

    Ok(())
}

pub struct Api {
    pub index_manager: IndexManager,
}

#[OpenApi]
impl Api {
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
    pub async fn v1_list_indexes(&self) -> ListIndexesResponse {
        v1_list_indexes(&self.index_manager)
    }

    #[oai(path = "/v1/indexes/:index", method = "get")]
    /// Get extended information about an index
    pub async fn v1_get_index(&self, index: Path<String>) -> GetIndexResponse {
        v1_get_index(&self.index_manager, index.to_string())
    }

    // v1/indexes/inventory_items_v1?schema=v1_descriptors
    #[oai(path = "/v1/indexes/:index", method = "post")]
    /// Create an index
    pub async fn v1_post_index(
        &self,
        index: Path<String>,
        schema: Query<String>,
        sort_by: Query<Option<String>>,
    ) -> PostIndexResponse {
        v1_post_index(
            &self.index_manager,
            index.to_string(),
            schema.0,
            sort_by.0.as_deref(),
        )
    }

    #[oai(path = "/v1/indexes/:index", method = "delete")]
    /// Delete an index
    pub async fn v1_delete_index(&self, index: Path<String>) -> DeleteIndexResponse {
        v1_delete_index(&self.index_manager, index.to_string())
    }

    #[oai(path = "/v1/indexes/:index/bulk_index", method = "post")]
    /// Index a list of SMILES and associated, free-form JSON attributes
    /// which are indexed and searchable
    pub async fn v1_post_indexes_bulk_index(
        &self,
        index: Path<String>,
        bulk_request: Json<BulkRequest>,
    ) -> PostIndexesBulkIndexResponse {
        v1_post_index_bulk(&self.index_manager, index.to_string(), bulk_request.0).await
    }

    #[oai(path = "/v1/indexes/:index/bulk_delete", method = "delete")]
    /// Delete a list of smiles (after standardization) from an index
    pub async fn v1_delete_indexes_bulk_delete(
        &self,
        index: Path<String>,
        bulk_request: Json<BulkRequest>,
    ) -> DeleteIndexesBulkDeleteResponse {
        v1_delete_index_bulk(&self.index_manager, index.to_string(), bulk_request.0).await
    }

    #[oai(path = "/v1/indexes/:index/search/basic", method = "get")]
    /// Perform basic query search against index
    pub async fn v1_index_search_basic(
        &self,
        index: Path<String>,
        query: Query<String>,
        limit: Query<Option<usize>>,
    ) -> GetQuerySearchResponse {
        let limit = if let Some(limit) = limit.0 {
            limit
        } else {
            usize::try_from(1000).unwrap()
        };

        v1_index_search_basic(&self.index_manager, index.to_string(), query.0, limit)
    }

    #[oai(path = "/v1/indexes/:index/search/substructure", method = "get")]
    /// Perform substructure search against index
    pub async fn v1_index_search_substructure(
        &self,
        index: Path<String>,
        smiles: Query<String>,
        result_limit: Query<Option<usize>>,
        tautomer_limit: Query<Option<usize>>,
        extra_query: Query<Option<String>>,
        use_scaffolds: Query<Option<String>>,
    ) -> GetStructureSearchResponse {
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

        let index = self.index_manager.open(&index);

        v1_index_search_structure(
            index,
            smiles.0,
            "substructure",
            result_limit,
            tautomer_limit,
            &extra_query,
            use_scaffolds,
        )
    }

    #[oai(path = "/v1/indexes/:index/search/superstructure", method = "get")]
    /// Perform superstructure search against index
    pub async fn v1_index_search_superstructure(
        &self,
        index: Path<String>,
        smiles: Query<String>,
        result_limit: Query<Option<usize>>,
        tautomer_limit: Query<Option<usize>>,
        extra_query: Query<Option<String>>,
        use_scaffolds: Query<Option<String>>,
    ) -> GetStructureSearchResponse {
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

        let index = self.index_manager.open(&index);

        v1_index_search_structure(
            index,
            smiles.0,
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
        extra_query: Query<Option<String>>,
        use_scaffolds: Query<Option<String>>,
    ) -> GetStructureSearchResponse {
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
            &self.index_manager,
            index.to_string(),
            smiles.0,
            &extra_query,
            use_scaffolds,
        )
    }

    #[oai(path = "/v1/indexes/:index/search/similarity", method = "get")]
    /// Perform descriptor/fingerprint-based similarity search against index
    pub async fn v1_index_search_similarity(
        &self,
        index: Path<String>,
        smiles: Query<String>,
        result_limit: Query<Option<usize>>,
        tautomer_limit: Query<Option<usize>>,
        bin_limit: Query<Option<usize>>,
        extra_query: Query<Option<String>>,
    ) -> GetStructureSearchResponse {
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

        let bin_limit = if let Some(bin_limit) = bin_limit.0 {
            bin_limit
        } else {
            usize::try_from(100).unwrap()
        };

        let extra_query = if let Some(extra_query) = extra_query.0 {
            extra_query
        } else {
            "".to_string()
        };

        let index = self.index_manager.open(&index);

        v1_index_search_similarity(
            index,
            smiles.0,
            result_limit,
            tautomer_limit,
            bin_limit,
            &extra_query,
        )
    }
}

pub fn output_spec(server_url: &str, output: &str) -> eyre::Result<()> {
    let api_service = api_service(
        server_url,
        API_PREFIX,
        std::path::PathBuf::from("/tmp/cheminee"),
        false,
    )?;

    let spec = api_service.spec();

    std::fs::write(output, spec)?;

    Ok(())
}
