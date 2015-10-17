use fractal::Beta;
use std::collections::VecDeque;

use traffic::{self, Traffic};
use {Config, Result, Source};

pub struct Fractal {
    time: f64,
    model: Beta,
    source: Source,
    arrivals: VecDeque<f64>,
}

impl Fractal {
    pub fn new(config: &Config, source: &Source) -> Result<Fractal> {
        let path = path!(config, "a traffic-pattern database is required");

        info!(target: "Traffic", "Reading {:?}...", &path);
        let data = try!(traffic::read_interarrivals(&path));
        let ncoarse = match (data.len() as f64).log2().floor() {
            ncoarse if ncoarse < 1.0 => raise!("there are not enough data"),
            ncoarse => ncoarse as usize,
        };

        info!(target: "Traffic", "Read {} arrivals. Fitting the model...", data.len());
        Ok(Fractal {
            time: 0.0,
            model: ok!(Beta::new(&data, ncoarse)),
            source: source.clone(),
            arrivals: VecDeque::new(),
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

macro_rules! refill(
    ($this:ident) => (
        if $this.arrivals.is_empty() {
            if let Err(error) = $this.refill() {
                error!(target: "Traffic", "Failed to refill the queue ({}).", error);
                return None;
            }
        }
    );
);

impl Traffic for Fractal {
    fn next(&mut self) -> Option<f64> {
        refill!(self);
        self.arrivals.pop_front()
    }

    fn peek(&mut self) -> Option<&f64> {
        refill!(self);
        self.arrivals.get(0)
    }
}
