pub fn generate_gamma_lut(gamma: f32, contrast: f32, brightness: f32) -> [u8; 256] {
    let mut lut = [0u8; 256];
    let inv_gamma = 1.0 / gamma;

    for i in 0..256 {
        let mut val = (i as f32 * contrast) + brightness;
        val = val.clamp(0.0, 255.0);

        let gamma_corrected = ((val / 255.0).powf(inv_gamma) * 255.0) as u32;
        lut[i] = gamma_corrected.min(255) as u8;
    }

    lut
}
