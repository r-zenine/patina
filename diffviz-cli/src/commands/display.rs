use anyhow::Result;
use diffviz_review::summarize_review_state;
use std::path::Path;

pub struct DisplayReviewCommand {
    folder: String,
    full: bool,
}

impl DisplayReviewCommand {
    pub fn new(folder: String, full: bool) -> Self {
        Self { folder, full }
    }

    pub fn run(&self) -> Result<()> {
        let folder = Path::new(&self.folder);
        let summary = summarize_review_state(folder)
            .map_err(|e| anyhow::anyhow!("Failed to summarize review state: {e}"))?;
        let yaml = if self.full {
            summary.to_yaml_full()
        } else {
            summary.to_yaml_minimal()
        }
        .map_err(|e| anyhow::anyhow!("Failed to serialize to YAML: {e}"))?;
        print!("{yaml}");
        Ok(())
    }
}
