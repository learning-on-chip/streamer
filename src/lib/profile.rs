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
        let (s1, s2) = (self.time_step, time_step);
        let end_time = t2 + (data.len() as f64) * s2;

        let steps = ((end_time - t1) / s1).ceil() as usize;
        if steps > self.steps {
            self.data.extend(vec![0.0; (steps - self.steps) * self.units]);
            self.steps = steps;
        }

        let offset = (time / s2).fract();
        let mut time = t2;
        while time < end_time {
            let l1 = (time / s1).floor() * s1;
            let l2 = offset + ((time - offset) / s2).floor() * s2;

            let r1 = l1 + s1;
            let r2 = l2 + s2;

            let weight = if l1 <= l2 {
                if r2 <= r1 { 1.0 } else { (r1 - l2) / s2 }
            } else {
                if r1 <= r2 { s1 / s2 } else { (r2 - l1) / s2 }
            };

            let j1 = ((time - t1) / s1) as usize;
            let j2 = ((time - t2) / s2) as usize;
            self.data[j1 * self.units + unit] += weight * data[j2];

            time = r1.min(r2);
        }
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
}
