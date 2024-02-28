use poem_openapi_derive::Object;

#[derive(Object, Debug)]
pub struct MolBlock {
    pub mol_block: String,
}
