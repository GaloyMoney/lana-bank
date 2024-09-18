use cloud_storage::Object;

use super::{ReportError, ReportLocationInCloud};

pub(super) async fn generate_download_link(
    location: &ReportLocationInCloud,
    duration_in_secs: u32,
) -> Result<String, ReportError> {
    Ok(Object::read(&location.bucket, &location.path_in_bucket)
        .await?
        .download_url(duration_in_secs)?)
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
