use crate::globals::consts::FMODEL_MAGIC;
use std::fs;

/// Reads the models persisted in the user's `.fmodel` file.
///
/// Metadata such as quantity of models and model size are read from the header section.
/// Each subsequent object queried from the file is then converted from Little Endiann bytes into 
/// floating point numbers.
///
/// ### Params:
/// @username: The user model to read in the system.
///
/// ### Returns:
/// A list containing all reconstructed models.
pub fn parse(username: &str) -> Vec<Vec<f32>> {
    let mut models = Vec::new();
    let data = fs::read(format!("/etc/facescape/{}.fmodel", username)).expect(&format!(
        "[FaceScape] Model file not found for {}",
        username
    ));

    assert!(&data[..4] == FMODEL_MAGIC, "[FaceScape] Invalid model file");

    let n_models = data[4] as usize;
    let model_size = u16::from_le_bytes(data[5..7].try_into().unwrap()) as usize;
    let expected_binary_bytes = model_size * 4;

    let mut cursor = 7;

    for _ in 0..n_models {
        let raw = &data[cursor..cursor + expected_binary_bytes];

        let model = raw
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
            .collect();

        models.push(model);
        
        cursor += expected_binary_bytes;
    }

    models
}
