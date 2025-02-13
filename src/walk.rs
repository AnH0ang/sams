use std::path::Path;

use anyhow::Result;
use ignore::overrides::{Override, OverrideBuilder};
use ignore::{Walk, WalkBuilder};

pub struct WalkOptions {
    /// Filter files by extension
    pub filter_extension: Option<String>,

    /// Exclude files by extension
    pub excludes: Vec<String>,

    /// Ignore hidden files
    pub ignore_hidden: bool,

    /// Ignore gitignore files
    pub respect_gitignore: bool,
}

impl Default for WalkOptions {
    fn default() -> Self {
        Self {
            filter_extension: None,
            excludes: Vec::new(),
            ignore_hidden: false,
            respect_gitignore: true,
        }
    }
}

impl WalkOptions {
    pub fn walk(self, root: &Path) -> Result<Walk> {
        let mut builder = WalkBuilder::new(root);

        builder.standard_filters(self.respect_gitignore);
        builder.hidden(self.ignore_hidden);
        builder.filter_entry(|entry| entry.file_type().is_some_and(|ft| ft.is_file()));

        let overrides = self.build_glob(root)?;
        builder.overrides(overrides);

        if let Some(ext) = self.filter_extension {
            builder.filter_entry(move |entry| {
                entry
                    .path()
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| e == ext)
            });
        };

        Ok(builder.build())
    }

    fn build_glob(&self, root: &Path) -> Result<Override> {
        let mut builder = OverrideBuilder::new(root);
        builder.add("!.git/")?;
        for glob in &self.excludes {
            builder.add(&format!("!{}", glob))?;
        }
        Ok(builder.build()?)
    }
}
