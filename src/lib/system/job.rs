use workload::Pattern;

/// A job.
#[derive(Clone, Debug)]
pub struct Job {
    /// The identifier.
    pub id: usize,
    /// The arrival time.
    pub arrival: f64,
    /// The workload pattern.
    pub pattern: Pattern,
}

impl Job {
    /// Create a job.
    #[inline]
    pub fn new(id: usize, arrival: f64, pattern: Pattern) -> Job {
        Job { id: id, arrival: arrival, pattern: pattern }
    }
}
