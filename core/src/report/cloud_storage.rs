use cloud_storage::Object;

use super::{ReportError, ReportLocationInCloud};

const LINK_DURATION_IN_SECS: u32 = 60 * 5;

pub(super) async fn generate_download_link(
    location: &ReportLocationInCloud,
) -> Result<String, ReportError> {
    Ok(Object::read(&location.bucket, &location.path_in_bucket)
        .await?
        .download_url(LINK_DURATION_IN_SECS)?)
}

pub(super) async fn upload_xml_file(
    location: &ReportLocationInCloud,
    xml_file: Vec<u8>,
) -> Result<(), ReportError> {
    Object::create(
        &location.bucket,
        xml_file,
        &location.path_in_bucket,
        "application/xml",
    )
    .await?;
    Ok(())
}
