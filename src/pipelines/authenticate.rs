/// Authenticator holds the pipeline for scoring likeness between two models and match against a
/// threshold.
pub struct Authenticator {
    int_model: Vec<f32>,
    part_model: Option<Vec<f32>>,
    likeness: f32,
    threshold: f32,
}

impl Authenticator {
    /// Create a new authentication pipeline from a internal model.
    ///
    /// The threshold must me lower than 1.0 and higher than 0.0.
    ///
    /// ### Params: 
    /// @int_model: The resident facial model of the user. \
    /// @threshold: The value to match against the likeness.
    ///
    /// ### Returns:
    /// An instance of Self.
    pub fn new(int_model: Vec<f32>, threshold: f32) -> Self {
        if threshold > 1.0 || threshold < 0.0 {
            panic!("Threshold must not be declared outside score bounds.")
        };

        Self {
            int_model,
            part_model: None,
            likeness: 0.0,
            threshold
        }
    }

    /// Sets a challenger on the pipeline to compare against the resident model.
    ///
    /// ### Params:
    /// @part_model: The partial model to authenticate against.
    pub fn challenger(&mut self, part_model: Vec<f32>) {
        self.part_model = Some(part_model);
    }

    /// Calculates the cosine similarity between the two defined models in the struct.
    ///
    /// It first defines the dot product between the two models. Then it calculates the individual
    /// magnitude of each model, gets the square root of both magnitudes and divides the dot product
    /// by the magnitude multiplied from both models.
    ///
    /// The similarity score is then stored inside of the property self.likeness
    pub fn cosine_similarity(&mut self) {
        let part_model = self.part_model.take().unwrap();

        println!("Model 1 Sum: {}, Model 2 Sum: {}", self.int_model.iter().sum::<f32>(), part_model.iter().sum::<f32>());
        let dot_product: f32 = self.int_model.iter()
            .zip(part_model.iter())
            .map(|(a, b)| a * b)
            .sum();

        let magnitute_int: f32 = self.int_model.iter()
            .copied()
            .map(|x| x * x)
            .sum();

        let magnitute_part: f32 = part_model.iter()
            .copied()
            .map(|x| x * x)
            .sum();

        println!("Model 1 Mag: {}, Model 2 Mag: {}", magnitute_int, magnitute_part);

        let mul_magnitude: f32 = magnitute_int.sqrt() * magnitute_part.sqrt();
        let cosine_similarity: f32;

        if mul_magnitude > 0.0 {
            cosine_similarity = dot_product / mul_magnitude;
        } else {
            cosine_similarity = 0.0;
        };

        self.likeness = cosine_similarity
    }

    /// Matches the similarity score against a threshold. If the likeness is higher than the
    /// threshold, it returns true.
    pub fn match_similarity(&self) -> bool {
        self.likeness > self.threshold
    }

    /// Returns the likeness score of the match.
    pub fn likeness(&self) -> &f32 {
        &self.likeness
    }
}
