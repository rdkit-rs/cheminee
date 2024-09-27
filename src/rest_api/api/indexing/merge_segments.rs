use poem_openapi::payload::Json;
use tantivy::TantivyDocument;

use crate::{indexing::index_manager::IndexManager, rest_api::api::MergeSegmentsResponse};

pub async fn v1_merge_segments(
    index_manager: &IndexManager,
    index: String,
) -> MergeSegmentsResponse {
    let index = index_manager.open(&index);

    let index = match index {
        Ok(i) => i,
        Err(_) => return MergeSegmentsResponse::IndexDoesNotExist,
    };

    let segments = index.searchable_segment_ids();
    let segments = match segments {
        Ok(s) => s,
        Err(e) => {
            return MergeSegmentsResponse::MergeFailed(Json(format!(
                "could not get segments: {:?}",
                e
            )))
        }
    };

    let writer = index.writer::<TantivyDocument>(64 * 1024 * 1024);

    let mut writer = match writer {
        Ok(w) => w,
        Err(e) => {
            return MergeSegmentsResponse::MergeFailed(Json(format!(
                "could not build writer: {:?}",
                e
            )))
        }
    };

    let merge_operation = writer.merge(&segments).wait();
    // .wait_merging_threads()
    // .await;

    match merge_operation {
        Ok(_) => (),
        Err(e) => {
            return MergeSegmentsResponse::MergeFailed(Json(format!("merge failed: {:?}", e)))
        }
    }

    MergeSegmentsResponse::Ok(Json("donezo".into()))
}
