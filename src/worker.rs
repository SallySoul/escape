use std::sync::Arc;

use crate::config::RenderConfig;
use crate::view_config::ViewConfig;
use crate::types::{Complex, CountGrid};

struct WorkerState {
    render_config: RenderConfig,
    view_config: ViewConfig,
    grids: Vec<Arc<CountGrid>>,
}

impl WorkerState {
    fn record_orbit(&self, orbit: &[Complex]) -> bool {
        for (i, cutoff) in self.render_config.cutoffs.iter().enumerate() {
            if orbit.len() <= cutoff.cutoff {
                return self.record_orbit_to_grid(orbit, &self.grids[i]);
            }
        }
        false
    }

    fn record_orbit_to_grid(&self, orbit: &[Complex], grid: &CountGrid) -> bool {
        let result = false;
        for c in orbit {
            if let Some((x, y)) = self.view_config.project(c) {
                grid.increment(x, y);
                result = true;
            }
        }
        result
    }
}

/*

async fn run_worker(state: Arc<WorkerState>) -> EscapeResult {
    Ok(())
}
*/
