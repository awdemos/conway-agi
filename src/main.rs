use anyhow::Result;
use conway_agi::simulation::Simulation;
use conway_agi::tui::run_app;

fn main() -> Result<()> {
    let sim = Simulation::new(120, 60).with_seed(0.25);
    run_app(sim)
}
