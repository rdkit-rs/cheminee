use poem_openapi_derive::Object;

#[derive(Object, Debug)]
pub struct Smiles {
    pub smiles: String,
}
