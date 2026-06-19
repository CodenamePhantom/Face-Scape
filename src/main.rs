mod core;
mod globals;
mod model;
mod pipelines;

use crate::pipelines::construct::Constructor;
use crate::pipelines::authenticate::Authenticator;
use crate::core::webcam_controller::WebcamIngress;
use crate::core::fourier_engine::FourierFaceEngine;

fn main() {
    let webcam = WebcamIngress::new("/dev/video0", Some((720, 1280))).unwrap();

    let (w, h) = webcam.resolution();
    let fourier_engine = FourierFaceEngine::new(w as usize, h as usize);

    let centroid_one = Constructor::run(&webcam, &fourier_engine);
    let centroid_two = Constructor::run(&webcam, &fourier_engine);
    let auth = Authenticator::new(centroid_one, centroid_two);

    let similarity = auth.cosine_similarity();

    if similarity < 0.95 {
        println!("Auth failed :( Likeness: {:.20}", similarity)
    } else {
        println!("Welcome! Likeness: {:.20}", similarity)
    }
}
