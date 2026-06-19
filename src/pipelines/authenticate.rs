pub struct Authenticator {
    int_model: Vec<f32>,
    part_model: Vec<f32>,
}

impl Authenticator {

    pub fn new(int_model: Vec<f32>, part_model: Vec<f32>) -> Self {
        Self {
            int_model,
            part_model
        }
    }

    pub fn cosine_similarity(&self) -> f32 {
        println!("Model 1 Sum: {}, Model 2 Sum: {}", self.int_model.iter().sum::<f32>(), self.part_model.iter().sum::<f32>());
        let dot_product: f32 = self.int_model.iter()
            .zip(self.part_model.iter())
            .map(|(a, b)| a * b)
            .sum();

        let magnitute_int: f32 = self.int_model.iter()
            .copied()
            .map(|x| x * x)
            .sum();

        let magnitute_part: f32 = self.part_model.iter()
            .copied()
            .map(|x| x * x)
            .sum();

        println!("Model 1 Mag: {}, Model 2 Mag: {}", magnitute_int, magnitute_part);

        let mul_magnitude: f32 = magnitute_int.sqrt() * magnitute_part.sqrt();

        if mul_magnitude > 0.0 {
            dot_product / mul_magnitude
        } else {
            0.0
        }
    }
}
