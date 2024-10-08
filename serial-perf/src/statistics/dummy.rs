use super::Statistics;

/// Dummy statistics used to disable the statistics for the path
#[derive(Debug, Default)]
pub struct DummyStatistics;

impl Statistics for DummyStatistics {
    fn add_failed(&mut self, _count: usize) {
        // Do nothing
    }

    fn add_successful(&mut self, _count: usize) {
        // Do nothing
    }

    fn reset(&mut self) {
        // Do nothing
    }
}
