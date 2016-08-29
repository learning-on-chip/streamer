use std::mem;

/// A platform profile.
///
/// A profile is a matrix that captures the evolution of a parameter over a time
/// interval with respect to a number of processing elements.
pub struct Profile {
    /// The number of processing elements.
    pub units: usize,
    /// The number of time steps.
    pub steps: usize,
    /// The beginning of the time interval.
    pub time: f64,
    /// The time step (sampling interval).
    pub time_step: f64,
    /// The actual data.
    pub data: Vec<f64>,
}

/// A builder of platform profiles.
pub struct ProfileBuilder {
    profile: Profile,
    fill: Vec<f64>,
}

impl Profile {
    /// Create a profile.
    #[inline]
    pub fn new(units: usize, time_step: f64) -> Profile {
        Profile { units: units, steps: 0, time: 0.0, time_step: time_step, data: vec![] }
    }

    /// Create a copy but with the data zeroed out.
    pub fn clone_zero(&self) -> Profile {
        Profile {
            units: self.units,
            steps: self.steps,
            time: self.time,
            time_step: self.time_step,
            data: vec![0.0; self.units * self.steps],
        }
    }

    fn extend(&mut self, steps: usize, fill: &[f64]) {
        debug_assert_eq!(fill.len(), self.units);
        self.data.reserve(steps * self.units);
        for _ in 0..steps {
            self.data.extend(fill);
        }
        self.steps += steps;
    }
}

impl ProfileBuilder {
    /// Create a builder.
    #[inline]
    pub fn new(units: usize, time_step: f64, fill: Vec<f64>) -> ProfileBuilder {
        debug_assert_eq!(units, fill.len());
        ProfileBuilder { profile: Profile::new(units, time_step), fill: fill }
    }

    /// Add data to a particular processing element starting from a particular
    /// time moment.
    pub fn push(&mut self, unit: usize, time: f64, time_step: f64, data: &[f64]) {
        let &mut ProfileBuilder { ref mut profile, ref fill } = self;
        debug_assert!(unit < profile.units);
        debug_assert!(time >= profile.time);
        let (t1, t2) = (profile.time, time);
        let (d1, d2) = (profile.time_step, time_step);
        let s2 = data.len();
        let s1 = ((t2 - t1 + (s2 as f64) * d2) / d1).ceil() as usize;
        if s1 > profile.steps {
            let more = s1 - profile.steps;
            profile.extend(more, fill);
        }
        let mut j1 = ((t2 - t1) / d1) as usize;
        let mut j2 = 0;
        macro_rules! add(
            ($weight:expr) => (profile.data[j1 * profile.units + unit] += $weight * data[j2]);
        );
        while j1 < s1 && j2 < s2 {
            let l1 = t1 + (j1 as f64) * d1;
            let l2 = t2 + (j2 as f64) * d2;
            let r1 = l1 + d1;
            let r2 = l2 + d2;
            if l1 < l2 {
                if r2 < r1 {
                    add!(1.0);
                    j2 += 1;
                } else {
                    add!((r1 - l2) / d2);
                    j1 += 1;
                }
            } else {
                if r1 < r2 {
                    add!(d1 / d2);
                    j1 += 1;
                } else {
                    add!((r2 - l1) / d2);
                    j2 += 1;
                }
            }
        }
    }

    /// Advance time and return the data accumulated since the previous call.
    pub fn pull(&mut self, time: f64) -> Profile {
        let &mut ProfileBuilder { ref mut profile, ref fill } = self;
        debug_assert!(time >= profile.time);
        let steps = ((time - profile.time) / profile.time_step).floor() as usize;
        if profile.steps < steps {
            let more = steps - profile.steps;
            profile.extend(more, fill);
        }
        let mut another = Profile {
            units: profile.units,
            steps: profile.steps - steps,
            time: (time / profile.time_step).floor() * profile.time_step,
            time_step: profile.time_step,
            data: profile.data[(steps * profile.units)..].to_vec(),
        };
        mem::swap(profile, &mut another);
        another.steps = steps;
        another.data.truncate(steps * profile.units);
        another
    }
}

impl Into<Vec<f64>> for Profile {
    #[inline]
    fn into(self) -> Vec<f64> {
        self.data
    }
}

deref! { Profile::data => [f64] }
deref! { mut Profile::data => [f64] }

#[cfg(test)]
mod tests {
    use super::ProfileBuilder;

    macro_rules! eq(
        (&$builder:ident.$field:ident, $value:expr) => (
            assert_eq!(&$builder.profile.$field, $value);
        );
        ($builder:ident.$field:ident, $value:expr) => (
            assert_eq!($builder.profile.$field, $value);
        );
        ($left:expr, $right:expr) => (
            assert_eq!($left, $right);
        );
    );

    #[test]
    fn push_fill() {
        let mut builder = ProfileBuilder::new(2, 0.5, vec![42.0, 42.0]);

        builder.push(0, 4.0, 1.0, &[]);
        eq!(builder.steps, 8);
        eq!(&builder.data, &vec![42.0; 2 * 8]);

        builder.push(0, 6.5, 1.0, &[]);
        eq!(builder.steps, 13);
        eq!(&builder.data, &vec![42.0; 2 * 13]);

        builder.push(0, 6.55, 1.0, &[]);
        eq!(builder.steps, 14);
        eq!(&builder.data, &vec![42.0; 2 * 14]);

        let mut builder = ProfileBuilder::new(2, 1.0, vec![42.0, 69.0]);

        builder.push(0, 0.0, 1.0, &[8.0, 8.0]);
        eq!(builder.steps, 2);
        eq!(&builder.data, &vec![50.0, 69.0, 50.0, 69.0]);

        builder.push(1, 1.0, 1.0, &[1.0, 1.0]);
        eq!(builder.steps, 3);
        eq!(&builder.data, &vec![50.0, 69.0, 50.0, 70.0, 42.0, 70.0]);
    }

    #[test]
    fn push_synchronous() {
        let mut builder = ProfileBuilder::new(2, 1.0, vec![0.0, 0.0]);

        builder.push(0, 1.0, 1.0, &[1.0, 2.0]);
        eq!(builder.steps, 3);
        eq!(&builder.data, &vec![
           0.0, 0.0,
           1.0, 0.0,
           2.0, 0.0,
        ]);

        builder.push(0, 1.0, 1.0, &[1.0, 2.0, 3.0]);
        eq!(builder.steps, 4);
        eq!(&builder.data, &vec![
           0.0, 0.0,
           2.0, 0.0,
           4.0, 0.0,
           3.0, 0.0,
        ]);
    }

    #[test]
    fn push_asynchronous() {
        let mut builder = ProfileBuilder::new(2, 1.0, vec![0.0, 0.0]);

        builder.push(1, 1.5, 1.0, &[1.0, 2.0, 3.0]);
        eq!(builder.steps, 5);
        eq!(&builder.data, &vec![
           0.0, 0.0,
           0.0, 0.5,
           0.0, 1.5,
           0.0, 2.5,
           0.0, 1.5,
        ]);

        builder.push(0, 0.5, 0.25, &[1.0, 2.0, 3.0, 1.0, 3.0]);
        eq!(builder.steps, 5);
        eq!(&builder.data, &vec![
           3.0, 0.0,
           7.0, 0.5,
           0.0, 1.5,
           0.0, 2.5,
           0.0, 1.5,
        ]);

        builder.push(0, 1.25, 1.0, &[1.0, 2.0, 3.0, 0.0, 4.0]);
        eq!(builder.steps, 7);
        eq!(&builder.data, &vec![
           3.00, 0.0,
           7.75, 0.5,
           1.75, 1.5,
           2.75, 2.5,
           0.75, 1.5,
           3.00, 0.0,
           1.00, 0.0,
        ]);
    }

    #[test]
    fn pull() {
        let mut builder = ProfileBuilder::new(2, 1.0, vec![0.0, 0.0]);
        builder.push(0, 0.0, 1.0, &vec![42.0; 42]);
        eq!(builder.time, 0.0);
        eq!(builder.steps, 42);

        eq!(builder.pull(0.0).data, vec![]);
        eq!(builder.time, 0.0);
        eq!(builder.steps, 42);

        eq!(builder.pull(0.75).data, vec![]);
        eq!(builder.time, 0.0);
        eq!(builder.steps, 42);

        eq!(builder.pull(1.0).data, vec![42.0, 0.0]);
        eq!(builder.time, 1.0);
        eq!(builder.steps, 41);

        eq!(builder.pull(1.5).data, vec![]);
        eq!(builder.time, 1.0);
        eq!(builder.steps, 41);

        eq!(builder.pull(3.5).data, vec![42.0, 0.0, 42.0, 0.0]);
        eq!(builder.time, 3.0);
        eq!(builder.steps, 39);
    }
}
