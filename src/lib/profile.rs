use std::mem;
use std::ops::{Deref, DerefMut};

pub struct Profile {
    pub units: usize,
    pub steps: usize,
    pub time: f64,
    pub time_step: f64,
    pub data: Vec<f64>,
}

impl Profile {
    pub fn new(units: usize, time_step: f64) -> Profile {
        Profile { units: units, steps: 0, time: 0.0, time_step: time_step, data: vec![] }
    }

    pub fn clone_zero(&self) -> Profile {
        Profile {
            units: self.units,
            steps: self.steps,
            time: self.time,
            time_step: self.time_step,
            data: vec![0.0; self.units * self.steps],
        }
    }

    pub fn push(&mut self, unit: usize, time: f64, time_step: f64, dynamic: &[f64], statik: f64) {
        debug_assert!(unit < self.units);
        debug_assert!(time >= self.time);

        let (t1, t2) = (self.time, time);
        let (d1, d2) = (self.time_step, time_step);

        let s2 = dynamic.len();
        let s1 = ((t2 - t1 + (s2 as f64) * d2) / d1).ceil() as usize;

        if s1 > self.steps {
            self.data.extend(vec![statik; (s1 - self.steps) * self.units]);
            self.steps = s1;
        }

        let mut j1 = ((t2 - t1) / d1) as usize;
        let mut j2 = 0;

        macro_rules! add(
            ($weight:expr) => (self.data[j1 * self.units + unit] += $weight * dynamic[j2]);
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

    pub fn pull(&mut self, time: f64) -> Self {
        debug_assert!(time >= self.time);
        let steps = ((time - self.time) / self.time_step).floor() as usize;
        debug_assert!(self.steps >= steps);

        let mut profile = Profile {
            units: self.units,
            steps: self.steps - steps,
            time: (time / self.time_step).floor() * self.time_step,
            time_step: self.time_step,
            data: self.data[(steps * self.units)..].to_vec(),
        };

        mem::swap(self, &mut profile);

        profile.steps = steps;
        profile.data.truncate(steps * self.units);

        profile
    }
}

impl Into<Vec<f64>> for Profile {
    #[inline]
    fn into(self) -> Vec<f64> {
        self.data
    }
}

impl Deref for Profile {
    type Target = [f64];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for Profile {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[cfg(test)]
mod tests {
    use super::Profile;

    #[test]
    fn push_padding() {
        let mut profile = Profile::new(2, 0.5);

        profile.push(0, 4.0, 1.0, &[], 42.0);
        assert_eq!(profile.steps, 8);
        assert_eq!(&profile.data, &vec![42.0; 2 * 8]);

        profile.push(0, 2.5, 1.0, &[], 0.0);
        assert_eq!(profile.steps, 8);
        assert_eq!(&profile.data, &vec![42.0; 2 * 8]);

        profile.push(0, 6.5, 1.0, &[], 42.0);
        assert_eq!(profile.steps, 13);
        assert_eq!(&profile.data, &vec![42.0; 2 * 13]);

        profile.push(0, 6.55, 1.0, &[], 42.0);
        assert_eq!(profile.steps, 14);
        assert_eq!(&profile.data, &vec![42.0; 2 * 14]);
    }

    #[test]
    fn push_synchronous() {
        let mut profile = Profile::new(2, 1.0);

        profile.push(0, 1.0, 1.0, &[1.0, 2.0], 0.0);
        assert_eq!(profile.steps, 3);
        assert_eq!(&profile.data, &vec![
           0.0, 0.0,
           1.0, 0.0,
           2.0, 0.0,
        ]);

        profile.push(0, 1.0, 1.0, &[1.0, 2.0, 3.0], 0.0);
        assert_eq!(profile.steps, 4);
        assert_eq!(&profile.data, &vec![
           0.0, 0.0,
           2.0, 0.0,
           4.0, 0.0,
           3.0, 0.0,
        ]);
    }

    #[test]
    fn push_asynchronous() {
        let mut profile = Profile::new(2, 1.0);

        profile.push(1, 1.5, 1.0, &[1.0, 2.0, 3.0], 0.0);
        assert_eq!(profile.steps, 5);
        assert_eq!(&profile.data, &vec![
           0.0, 0.0,
           0.0, 0.5,
           0.0, 1.5,
           0.0, 2.5,
           0.0, 1.5,
        ]);

        profile.push(0, 0.5, 0.25, &[1.0, 2.0, 3.0, 1.0, 3.0], 0.0);
        assert_eq!(profile.steps, 5);
        assert_eq!(&profile.data, &vec![
           3.0, 0.0,
           7.0, 0.5,
           0.0, 1.5,
           0.0, 2.5,
           0.0, 1.5,
        ]);

        profile.push(0, 1.25, 1.0, &[1.0, 2.0, 3.0, 0.0, 4.0], 0.0);
        assert_eq!(profile.steps, 7);
        assert_eq!(&profile.data, &vec![
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
        let mut profile = Profile::new(2, 1.0);
        profile.push(0, 0.0, 1.0, &vec![42.0; 42], 0.0);
        assert_eq!(profile.time, 0.0);
        assert_eq!(profile.steps, 42);

        assert_eq!(profile.pull(0.0).data, vec![]);
        assert_eq!(profile.time, 0.0);
        assert_eq!(profile.steps, 42);

        assert_eq!(profile.pull(0.75).data, vec![]);
        assert_eq!(profile.time, 0.0);
        assert_eq!(profile.steps, 42);

        assert_eq!(profile.pull(1.0).data, vec![42.0, 0.0]);
        assert_eq!(profile.time, 1.0);
        assert_eq!(profile.steps, 41);

        assert_eq!(profile.pull(1.5).data, vec![]);
        assert_eq!(profile.time, 1.0);
        assert_eq!(profile.steps, 41);

        assert_eq!(profile.pull(3.5).data, vec![42.0, 0.0, 42.0, 0.0]);
        assert_eq!(profile.time, 3.0);
        assert_eq!(profile.steps, 39);
    }
}
