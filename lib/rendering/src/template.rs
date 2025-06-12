use handlebars::Handlebars;
use serde::Serialize;

use crate::error::RenderingError;

/// Template renderer for processing Handlebars templates
#[derive(Clone)]
pub struct TemplateRenderer {
    handlebars: Handlebars<'static>,
}

impl TemplateRenderer {
    /// Create a new template renderer
    pub fn new() -> Self {
        let handlebars = Handlebars::new();
        Self { handlebars }
    }

    /// Render a template string with the provided data
    pub fn render<T: Serialize>(
        &self,
        template_content: &str,
        data: &T,
    ) -> Result<String, RenderingError> {
        let rendered = self.handlebars.render_template(template_content, data)?;
        Ok(rendered)
    }
}

impl Default for TemplateRenderer {
    fn default() -> Self {
        Self::new()
    }
}
