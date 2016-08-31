use fractal::Beta;
use std::collections::VecDeque;

use {Config, Result, Source};
use traffic::{self, Traffic};

/// A multifractal wavelet model of network traffic.
pub struct Fractal {
    time: f64,
    model: Beta,
    arrivals: VecDeque<f64>,
    source: Source,
}

impl Fractal {
    /// Create a model.
    pub fn new(config: &Config, source: Source) -> Result<Fractal> {
        let path = path!(config, "a traffic-pattern database is required");
        info!(target: "Traffic", "Reading interarrivals from {:?}...", &path);
        let data = try!(traffic::read_interarrivals(&path));
        let ncoarse = match (data.len() as f64).log2().floor() {
            ncoarse if ncoarse < 1.0 => raise!("there are not enough data"),
            ncoarse => ncoarse as usize,
        };
        info!(target: "Traffic", "Read {} interarrivals. Fitting the model...", data.len());
        Ok(Fractal {
            time: 0.0,
            model: ok!(Beta::new(&data, ncoarse)),
            arrivals: VecDeque::new(),
            source: source,
        })
    }

    fn refill(&mut self) -> Result<()> {
        info!(target: "Traffic", "Refilling the queue...");
        for step in ok!(self.model.sample(&mut self.source)) {
            self.time += step;
            self.arrivals.push_back(self.time);
        }
        info!(target: "Traffic", "The queue contains {} arrivals.", self.arrivals.len());
        Ok(())
    }
}

impl Traffic for Fractal {
    fn next(&mut self) -> Result<Option<f64>> {
        if self.arrivals.is_empty() {
            try!(self.refill());
        }
        Ok(self.arrivals.pop_front())
    }

    fn peek(&mut self) -> Result<Option<&f64>> {
        if self.arrivals.is_empty() {
            try!(self.refill());
        }
        Ok(self.arrivals.get(0))
    }
}
