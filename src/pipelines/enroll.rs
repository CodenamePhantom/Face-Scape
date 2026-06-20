use crate::globals::consts::FMODEL_MAGIC;
use crate::pipelines::construct::Constructor;
use crate::core::webcam_controller::WebcamIngress;
use crate::core::fourier_engine::FourierFaceEngine;
use atomic_matrix::prelude::{ AtomicMatrix, MatrixHandler, uid_lite, memory_scale };
use std::io::{self, Write, BufWriter};
use std::fs::File;

pub struct Enroll {}

impl Enroll {
    pub fn enroll() {
        let mut enroll = Self {};
        let mut model_list = Vec::new();

        let webcam = WebcamIngress::new("/dev/video0", Some((720, 1280))).unwrap();
        let m_scale = memory_scale::custom::mb::<12>();

        let (w, h) = webcam.resolution();
        let fourier_engine = FourierFaceEngine::new(w as usize, h as usize);

        println!("[FaceScape] Enrollment protocol.");

        println!("[FaceScape - Step One] First, scan the front of your face from an average distance from your webcam.");
        println!("[FaceScape - Step One] Press enter to continue.");
        enroll.wait_input();

        let handler_one = enroll.generate_matrix(m_scale);
        let model_one = Constructor::run(&webcam, &fourier_engine, &handler_one);
        handler_one.die();

        println!("[FaceScape - Step Two] Now, get a bit closer.");
        println!("[FaceScape - Step Two] Press enter to continue.");
        enroll.wait_input();

        let handler_two = enroll.generate_matrix(m_scale);
        let model_two = Constructor::run(&webcam, &fourier_engine, &handler_two);
        handler_two.die();

        println!("[FaceScape - Step Three] A bit further now.");
        println!("[FaceScape - Step Three] Press enter to continue.");
        enroll.wait_input();

        let handler_three = enroll.generate_matrix(m_scale);
        let model_three = Constructor::run(&webcam, &fourier_engine, &handler_three);
        handler_three.die();

        println!("[FaceScape - Step Four] Persisting model.");
        println!("[FaceScape - Step Four] Generating interpolated frames.");
        let mid_1_2a = enroll.slerp(&model_one, &model_two, 0.33);
        let mid_1_2b = enroll.slerp(&model_one, &model_two, 0.66);

        let mid_2_3a = enroll.slerp(&model_two, &model_three, 0.33);
        let mid_2_3b = enroll.slerp(&model_two, &model_three, 0.66);

        println!("[FaceScape - Step Four] Persisting model.");
        model_list.push(model_one);
        model_list.push(mid_1_2a);
        model_list.push(mid_1_2b);
        model_list.push(model_two);
        model_list.push(mid_2_3a);
        model_list.push(mid_2_3b);
        model_list.push(model_three);

        enroll.persist_model(model_list);

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

    fn slerp(&self, a: &[f32], b: &[f32], t: f32) -> Vec<f32> {
        let dot = a.iter().zip(b).map(|(x, y)| x * y).sum::<f32>().clamp(-1.0, 1.0);
        let theta = dot.acos();

        if theta.abs() < 1e-6 {
            return a.to_vec();
        }

        let sin_theta = theta.sin();
        let wa = ((1.0 - t) * theta).sin() / sin_theta;
        let wb = (t * theta).sin() / sin_theta;

        a.iter().zip(b).map(|(x, y)| wa * x + wb * y).collect()
    }

    fn persist_model(&mut self, model_list: Vec<Vec<f32>>) {
        match std::fs::exists("/etc/facescape") {
            Ok(_) => {},
            Err(_) => { std::fs::create_dir("facescape").unwrap(); }
        }

        let file = File::create("/etc/facescape/user.fmodel").unwrap();

        let mut writer = BufWriter::new(file);

        writer.write_all(FMODEL_MAGIC).unwrap();
        writer.write_all(&[model_list.len() as u8]).unwrap();
        writer.write_all(&(model_list[0].len() as u16).to_le_bytes()).unwrap();


        for (i, model) in model_list.iter().enumerate() {
            let label = format!("model_{}\n", i + 1);

            writer.write_all("\n".as_bytes()).unwrap();
            writer.write_all(&label.as_bytes()).unwrap();
            
            for f in model {
                let bytes = f.to_le_bytes();
                writer.write_all(&bytes).unwrap();
            }
        };

        writer.flush().unwrap();
    }
}
