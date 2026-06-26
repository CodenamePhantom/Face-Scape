use crate::globals::consts::{ FMODEL_MAGIC, FOURIER_RADIUS };
use crate::pipelines::construct::Constructor;
use crate::core::webcam_controller::WebcamIngress;
use crate::core::fourier_engine::FourierFaceEngine;
use atomic_matrix::prelude::{ AtomicMatrix, MatrixHandler, uid_lite, memory_scale };
use std::io::{self, Write, BufWriter};
use std::fs::File;

pub struct Enroll {}

impl Enroll {
    pub fn enroll(user: String) {
        let mut enroll = Self {};

        let webcam = WebcamIngress::new("/dev/video0", Some((1920, 1080))).unwrap();
        let m_scale = memory_scale::custom::mb::<120>();

        let (w, h) = webcam.resolution();
        let fourier_engine = FourierFaceEngine::new(w as usize, h as usize);

        println!("[FaceScape] Enrollment protocol.");

        println!("[FaceScape - Step One] First, scan the front of your face far from your webcam.");
        println!("[FaceScape - Step One] Press enter to continue.");
        enroll.wait_input();

        let handler_one = enroll.generate_matrix(m_scale);
        let model_one = Constructor::run(&webcam, &fourier_engine, &handler_one, 30);
        handler_one.die();

        println!("[FaceScape - Step Two] Now, scan the front of your face from an average distance.");
        println!("[FaceScape - Step Two] Press enter to continue.");
        enroll.wait_input();

        let handler_two = enroll.generate_matrix(m_scale);
        let model_two = Constructor::run(&webcam, &fourier_engine, &handler_two, 30);
        handler_two.die();

        println!("[FaceScape - Step Three] Finally, scan the front of your face close to your webcam.");
        println!("[FaceScape - Step Three] Press enter to continue.");
        enroll.wait_input();

        let handler_three = enroll.generate_matrix(m_scale);
        let model_three = Constructor::run(&webcam, &fourier_engine, &handler_three, 30);
        handler_three.die();

        let multi_scale_models = Self::generate_interpolated_scale_space(&model_one, &model_two, &model_three);
        
        println!("[FaceScape - Step Four] Persisting model.");

        enroll.persist_model(multi_scale_models, user);

        println!("[FaceScape] Model persisted.");
    }

    fn generate_matrix(&self, m: usize) -> MatrixHandler {
        AtomicMatrix::bootstrap(
            Some(format!("fs_capture.{}", uid_lite::generate_uuid())), 
            m,
        ).unwrap()
    }
     
    fn wait_input(&self) {
        io::stdout().flush().unwrap();
        let mut _buffer = String::new();
        io::stdin().read_line(&mut _buffer).unwrap();
    }

    fn generate_interpolated_scale_space(
        v_far: &[f32],
        v_avg: &[f32],
        v_near: &[f32],
    ) -> Vec<Vec<f32>> {
        let mut scale_space = Vec::new();
        let steps_per_segment = 10;

        for i in 0..steps_per_segment {
            let t = i as f32 / steps_per_segment as f32;
            scale_space.push(Self::lerp_and_normalize(v_far, v_avg, t));
        }

        for i in 0..=steps_per_segment {
            let t = i as f32 / steps_per_segment as f32;
            scale_space.push(Self::lerp_and_normalize(v_avg, v_near, t));
        }

        scale_space
    }

    fn lerp_and_normalize(v1: &[f32], v2: &[f32], t: f32) -> Vec<f32> {
        let mut mixed = vec![0.0; v1.len()];
        let mut norm_sq = 0.0;

        for i in 0..v1.len() {
            mixed[i] = v1[i] + t *(v2[i] - v1[i]);
            norm_sq += mixed[i] * mixed[i];
        }

        let norm = norm_sq.sqrt();
        if norm > 1e-10 {
            mixed.iter_mut().for_each(|x| *x /= norm);
        }

        mixed
    }

    fn persist_model(&mut self, models: Vec<Vec<f32>>, user: String) {
        match std::fs::exists("/etc/facescape") {
            Ok(_) => {},
            Err(_) => { std::fs::create_dir("facescape").unwrap(); }
        }

        let file = File::create(format!("/etc/facescape/{}.fmodel", user)).unwrap();

        let mut writer = BufWriter::new(file);

        writer.write_all(FMODEL_MAGIC).unwrap();
        writer.write_all(&[models.len() as u8]).unwrap();
        writer.write_all(&(FOURIER_RADIUS as u16 * FOURIER_RADIUS as u16).to_le_bytes()).unwrap();

        for model in models {
            for f in model {
                let bytes = f.to_le_bytes();
                writer.write_all(&bytes).unwrap();
            }
        };

        writer.flush().unwrap();
    }
}
