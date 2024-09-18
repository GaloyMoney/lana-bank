use cloud_storage::Object;

use super::ReportError;

pub(super) async fn generate_download_link(
    bucket_name: &str,
    path_in_bucket: &str,
    duration_in_secs: u32,
) -> Result<String, ReportError> {
    Ok(Object::read(bucket_name, path_in_bucket)
        .await?
        .download_url(duration_in_secs)?)
}

pub(super) async fn upload_xml_file(
    bucket_name: &str,
    path_in_bucket: &str,
    xml_file: Vec<u8>,
) -> Result<(), ReportError> {
    Object::create(bucket_name, xml_file, path_in_bucket, "application/xml").await?;
    Ok(())
}
