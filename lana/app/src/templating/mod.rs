use anyhow::Result;
use handlebars::Handlebars;
use serde::Serialize;
use std::fs;
use uuid::Uuid;

pub fn from_markdown(markdown: &str) -> Result<Vec<u8>> {
    let temp_dir = std::env::temp_dir();
    let temp_file_name = temp_dir.join(format!("{}.pdf", Uuid::new_v4()));
    markdown2pdf::parse(markdown.to_string(), &temp_file_name.to_string_lossy())?;
    let pdf_bytes = fs::read(&temp_file_name)?;
    fs::remove_file(&temp_file_name)?;
    Ok(pdf_bytes)
}

pub fn create_pdf_from_template<T: Serialize>(template_path: &str, data: &T) -> Result<Vec<u8>> {
    let markdown_template = fs::read_to_string(template_path)?;
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("template", &markdown_template)?;
    let rendered_markdown = handlebars.render("template", data)?;
    from_markdown(&rendered_markdown)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_create_pdf_from_template() -> Result<()> {
        let mut data = BTreeMap::new();
        data.insert("name", "Test User");
        data.insert("amount", "$1000");

        let pdf_bytes =
            create_pdf_from_template("src/templating/templates/loan_agreement.md", &data)?;

        assert!(!pdf_bytes.is_empty());
        assert!(pdf_bytes.starts_with(b"%PDF"));

        // Create a directory for test outputs
        let output_dir = Path::new("test-output");
        fs::create_dir_all(output_dir)?;

        // Write the PDF to a file
        let output_path = output_dir.join("test_loan_agreement.pdf");
        fs::write(output_path, pdf_bytes)?;

        Ok(())
    }
}
