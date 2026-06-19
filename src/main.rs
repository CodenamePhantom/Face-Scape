mod core;
mod globals;
mod model;
mod pipelines;

use crate::pipelines::construct::Constructor;
use crate::pipelines::authenticate::Authenticator;
use crate::core::webcam_controller::WebcamIngress;
use crate::core::fourier_engine::FourierFaceEngine;
use atomic_matrix::prelude::{ AtomicMatrix, uid_lite, memory_scale };

fn main() {
    let webcam = WebcamIngress::new("/dev/video0", Some((720, 1280))).unwrap();

    let (w, h) = webcam.resolution();
    let fourier_engine = FourierFaceEngine::new(w as usize, h as usize);

    let m_scale = memory_scale::custom::gb::<2>();
    let handler_one = AtomicMatrix::bootstrap(
        Some(format!("fs_capture.{}", uid_lite::generate_uuid())),
        m_scale
    ).unwrap();
    let handler_two = AtomicMatrix::bootstrap(
        Some(format!("fs_auth.{}", uid_lite::generate_uuid())),
        m_scale
    ).unwrap();

    let centroid_one = Constructor::run(&webcam, &fourier_engine, &handler_one);
    handler_one.die();
    let centroid_two = Constructor::run(&webcam, &fourier_engine, &handler_two);
    handler_two.die();

    let auth = Authenticator::new(centroid_one, centroid_two);

    let similarity = auth.cosine_similarity();

    if similarity < 0.95 {
        println!("Auth failed :( Likeness: {:.20}", similarity)
    } else {
        println!("Welcome! Likeness: {:.20}", similarity)
    }
}
