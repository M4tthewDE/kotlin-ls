use std::path::PathBuf;

use anyhow::{Context, Result};
use tower_lsp::lsp_types::Hover;

use crate::{kotlin::Position, Backend};

impl Backend {
    pub fn get_hover(&self, path: &PathBuf, pos: &Position) -> Result<Option<Hover>> {
        let file = self
            .files
            .get(path)
            .with_context(|| format!("unknown path: {:?}", path))?;

        Ok(file.hover_element(pos))
    }
}
