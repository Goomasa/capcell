pub struct Medium {
    pub coeff_sc: f64,
    pub coeff_ab: f64,
}

impl Medium {
    pub fn new(coeff_sc: f64, coeff_ab: f64) -> Self {
        Medium { coeff_sc, coeff_ab }
    }
}
