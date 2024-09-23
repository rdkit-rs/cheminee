use poem_openapi::{payload::Json, ApiResponse, Object};

#[derive(ApiResponse, Debug)]
pub enum StandardizeResponse {
    #[oai(status = "200", content_type = "application/json")]
    Ok(Json<Vec<StandardizedSmiles>>),
}

#[derive(ApiResponse, Debug)]
pub enum ConvertedSmilesResponse {
    #[oai(status = "200", content_type = "application/json")]
    Ok(Json<Vec<ConvertedSmiles>>),
}

#[derive(ApiResponse, Debug)]
pub enum ConvertedMolBlockResponse {
    #[oai(status = "200", content_type = "application/json")]
    Ok(Json<Vec<ConvertedMolBlock>>),
}

#[derive(Object, Debug)]
pub struct StandardizedSmiles {
    #[oai(skip_serializing_if_is_none)]
    pub smiles: Option<String>,
    #[oai(skip_serializing_if_is_none)]
    pub error: Option<String>,
}

#[derive(Object, Debug)]
pub struct ConvertedSmiles {
    #[oai(skip_serializing_if_is_none)]
    pub smiles: Option<String>,
    #[oai(skip_serializing_if_is_none)]
    pub error: Option<String>,
}

#[derive(Object, Debug)]
pub struct ConvertedMolBlock {
    #[oai(skip_serializing_if_is_none)]
    pub mol_block: Option<String>,
    #[oai(skip_serializing_if_is_none)]
    pub error: Option<String>,
}
