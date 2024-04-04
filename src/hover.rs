use anyhow::{bail, Result};
use tower_lsp::lsp_types::{Hover, Position};

use crate::Backend;

impl Backend {
    pub fn get_hover(&self, _pos: &Position) -> Result<Option<Hover>> {
        bail!("todo: hover");
    }
}
