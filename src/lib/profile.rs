use std::mem;

pub struct Profile {
    units: usize,
    steps: usize,
    time: f64,
    time_step: f64,
    data: Vec<f64>,
}

impl Profile {
    pub fn new(units: usize, time_step: f64) -> Profile {
        Profile {
            units: units,
            steps: 0,
            time: 0.0,
            time_step: time_step,
            data: Vec::with_capacity(units),
        }
    }

    pub fn accumulate(&mut self, unit: usize, time: f64, time_step: f64, data: &[f64]) {
        debug_assert!(unit < self.units);
        debug_assert!(time >= self.time);

        let (t1, t2) = (self.time, time);
        let (d1, d2) = (self.time_step, time_step);

        let s2 = data.len();
        let s1 = ((t2 - t1 + (s2 as f64) * d2) / d1).ceil() as usize;

        if s1 > self.steps {
            self.data.extend(vec![0.0; (s1 - self.steps) * self.units]);
            self.steps = s1;
        }

        let mut j1 = ((t2 - t1) / d1) as usize;
        let mut j2 = 0;

        macro_rules! add(
            ($weight:expr) => (self.data[j1 * self.units + unit] += $weight * data[j2]);
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

    pub fn discharge(&mut self, time: f64) -> Vec<f64> {
        debug_assert!(time >= self.time);
        let steps = ((time - self.time) / self.time_step).floor() as usize;
        debug_assert!(self.steps >= steps);

        let tail = self.data[(steps * self.units)..].to_vec();
        let mut data = mem::replace(&mut self.data, tail);
        data.truncate(steps * self.units);

        self.steps = self.steps - steps;
        self.time = (time / self.time_step).floor() * self.time_step;

        data
    }
}

#[cfg(test)]
mod tests {
    use super::Profile;

    #[test]
    fn accumulate_padding() {
        let mut profile = Profile::new(2, 0.5);

        profile.accumulate(0, 4.0, 1.0, &[]);
        assert_eq!(profile.steps, 8);
        assert_eq!(&profile.data, &vec![0.0; 2 * 8]);

        profile.accumulate(0, 2.5, 1.0, &[]);
        assert_eq!(profile.steps, 8);
        assert_eq!(&profile.data, &vec![0.0; 2 * 8]);

        profile.accumulate(0, 6.5, 1.0, &[]);
        assert_eq!(profile.steps, 13);
        assert_eq!(&profile.data, &vec![0.0; 2 * 13]);

        profile.accumulate(0, 6.55, 1.0, &[]);
        assert_eq!(profile.steps, 14);
        assert_eq!(&profile.data, &vec![0.0; 2 * 14]);
    }

    #[test]
    fn accumulate_synchronous() {
        let mut profile = Profile::new(2, 1.0);

        profile.accumulate(0, 1.0, 1.0, &[1.0, 2.0]);
        assert_eq!(profile.steps, 3);
        assert_eq!(&profile.data, &vec![
           0.0, 0.0,
           1.0, 0.0,
           2.0, 0.0,
        ]);

        profile.accumulate(0, 1.0, 1.0, &[1.0, 2.0, 3.0]);
        assert_eq!(profile.steps, 4);
        assert_eq!(&profile.data, &vec![
           0.0, 0.0,
           2.0, 0.0,
           4.0, 0.0,
           3.0, 0.0,
        ]);
    }

    #[test]
    fn accumulate_asynchronous() {
        let mut profile = Profile::new(2, 1.0);

        profile.accumulate(1, 1.5, 1.0, &[1.0, 2.0, 3.0]);
        assert_eq!(profile.steps, 5);
        assert_eq!(&profile.data, &vec![
           0.0, 0.0,
           0.0, 0.5,
           0.0, 1.5,
           0.0, 2.5,
           0.0, 1.5,
        ]);

        profile.accumulate(0, 0.5, 0.25, &[1.0, 2.0, 3.0, 1.0, 3.0]);
        assert_eq!(profile.steps, 5);
        assert_eq!(&profile.data, &vec![
           3.0, 0.0,
           7.0, 0.5,
           0.0, 1.5,
           0.0, 2.5,
           0.0, 1.5,
        ]);

        profile.accumulate(0, 1.25, 1.0, &[1.0, 2.0, 3.0, 0.0, 4.0]);
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
    fn discharge() {
        let mut profile = Profile::new(2, 1.0);
        profile.accumulate(0, 0.0, 1.0, &vec![42.0; 42]);
        assert_eq!(profile.time, 0.0);
        assert_eq!(profile.steps, 42);

        assert_eq!(profile.discharge(0.0), vec![]);
        assert_eq!(profile.time, 0.0);
        assert_eq!(profile.steps, 42);

        assert_eq!(profile.discharge(0.75), vec![]);
        assert_eq!(profile.time, 0.0);
        assert_eq!(profile.steps, 42);

        assert_eq!(profile.discharge(1.0), vec![42.0, 0.0]);
        assert_eq!(profile.time, 1.0);
        assert_eq!(profile.steps, 41);

        assert_eq!(profile.discharge(1.5), vec![]);
        assert_eq!(profile.time, 1.0);
        assert_eq!(profile.steps, 41);

        assert_eq!(profile.discharge(3.5), vec![42.0, 0.0, 42.0, 0.0]);
        assert_eq!(profile.time, 3.0);
        assert_eq!(profile.steps, 39);
    }
}
