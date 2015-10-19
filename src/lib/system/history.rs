use system::{Event, EventKind};

/// Statistics about a system.
#[derive(Clone, Copy, Debug, Default)]
pub struct History {
    /// The number of created jobs.
    pub created: usize,
    /// The number of arrived jobs.
    pub arrived: usize,
    /// The number of started jobs.
    pub started: usize,
    /// The number of finished jobs.
    pub finished: usize,
}

impl History {
    /// Take into account an event.
    pub fn remember(&mut self, event: &Event) {
        match &event.kind {
            &EventKind::Arrived(..) => self.arrived += 1,
            &EventKind::Started(..) => self.started += 1,
            &EventKind::Finished(..) => self.finished += 1,
        }
    }
}
