mod core;
mod model;

use crate::core::{
    fourier_engine::FourierFaceEngine, 
    webcam_controller::WebcamIngress,
};
use std::time::{Duration, Instant};

pub fn main() {
    let mut fourier_frames = Vec::<Vec<f32>>::with_capacity(30);

    let webcam = WebcamIngress::new("/dev/video0", 720, 1280).unwrap();
    let (w, h) = webcam.resolution();
    let f_engine = FourierFaceEngine::new(w as usize, h as usize);

    let gs_frames = webcam.capture_gray_scale_frames(20).unwrap();

    let now = Instant::now();

    for frames in gs_frames {
        fourier_frames.push(f_engine.process_frame_to_coefficients(&frames.slice));
    }

    let elapsed_one = Duration::from_nanos(now.elapsed().as_nanos().try_into().unwrap());

    let centroid = FourierFaceEngine::centroid_frame_generator(fourier_frames);

    let elapsed_two = Duration::from_nanos(now.elapsed().as_nanos().try_into().unwrap());

    println!("Fourier map (With Gaussian Mask; Aggregated):");

    for y in 0..64 {
        for x in 0..64 {
            let v = centroid[y * 64 + x];
            let log_v = if v > 0.0 { v.ln() } else { 0.0 };

            let c = match log_v as u32 {
                11.. => '#',
                10.. => 'O',
                9.. => 'o',
                8.. => '*',
                7.. => '.',
                _ => ' ',
            };

            print!("{}", c);
        }

        println!();
    }

    println!(
        "Elapsed fourier transformation ({} frames): {:?}",
        30,
        elapsed_one
    );
    println!(
        "Elapsed linear median frame aggregation ({} frames): {:?}",
        30,
        elapsed_two - elapsed_one
    )
}
