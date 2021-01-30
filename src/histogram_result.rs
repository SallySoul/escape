use crate::config::SampleConfig;
use crate::error::{EscapeError, EscapeResult};
use crate::types::CountGrid;
use crate::types::NormalizedGrid;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone)]
pub struct HistogramResult {
    config: SampleConfig,
    grids: Vec<NormalizedGrid>,
}

impl HistogramResult {
    pub fn to_file(
        config: &SampleConfig,
        grids: &Vec<NormalizedGrid>,
        path: &std::path::Path,
    ) -> EscapeResult {
        let writer = BufWriter::new(std::fs::File::create(path)?);
        serde_json::to_writer(
            writer,
            &HistogramResult {
                config: config.clone(),
                grids: grids.clone(),
            },
        )?;
        Ok(())
    }

    pub fn from_file(
        path: &std::path::Path,
    ) -> Result<(Arc<SampleConfig>, Vec<NormalizedGrid>), EscapeError> {
        let reader = BufReader::new(std::fs::File::open(path)?);
        let partial_result: HistogramResult = serde_json::from_reader(reader)?;
        Ok((Arc::from(partial_result.config), partial_result.grids))
    }
}
