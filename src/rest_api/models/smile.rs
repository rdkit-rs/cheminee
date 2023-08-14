use poem_openapi_derive::Object;

#[derive(Object, Debug)]
pub struct Smile {
    pub smile: String,
}
