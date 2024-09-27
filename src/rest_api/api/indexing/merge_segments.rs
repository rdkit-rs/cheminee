use poem_openapi::payload::Json;

use crate::{
    indexing::{index_manager::IndexManager, segment_manager::SegmentManager},
    rest_api::api::MergeSegmentsResponse,
};

pub async fn v1_merge_segments(
    index_manager: &IndexManager,
    index: String,
) -> MergeSegmentsResponse {
    let index = index_manager.open(&index);

    let index = match index {
        Ok(i) => i,
        Err(_) => return MergeSegmentsResponse::IndexDoesNotExist,
    };

    let segment_manager = SegmentManager {};
    match segment_manager.merge(&index) {
        Ok(_) => (),
        Err(e) => return MergeSegmentsResponse::MergeFailed(Json(e.to_string())),
    }

    MergeSegmentsResponse::Ok(Json("donezo".into()))
}
