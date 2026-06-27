use crate::core::fourier_engine::FourierFaceEngine;
use crate::core::webcam_controller::WebcamIngress;
use crate::globals::consts::{FOURIER_RADIUS, INIT_FLAG, MODEL_ARENA, RESIDENT_MODEL};
use crate::pipelines::construct::Constructor;
use atomic_matrix::extensive_lib::looper::Looper;
use atomic_matrix::prelude::{
    AtomicMatrix, HEADER_SPACE, HandlerFunctions, MatrixHandler, RelativePtr, memory_scale,
    uid_lite,
};
use std::sync::atomic::Ordering;

/// Authenticator holds the pipeline for scoring likeness between two models and match against a
/// threshold.
///
/// I achieves that by loading the user models written in memory, capturing a fresh partial model
/// from the challenger caller, and running a cosine similarity between the part model and each
/// internal model found in the arena. The best matching model slice is then used for
/// authentication.
pub struct Authenticator {}

impl Authenticator {
    /// Runs the authentication pipeline.
    /// 
    /// ### Params:
    /// @user: The user to run auth against.
    pub fn run(user: String) {
        let handler = AtomicMatrix::bootstrap(
            Some(format!("{}.{}", MODEL_ARENA, user)),
            memory_scale::custom::mb::<20>(),
        )
        .unwrap();
        let mut cs_list = Vec::<f32>::new();

        let int_models = Self::get_int_models(&handler);
        let mut part_model = Self::get_part_model();
        part_model.rotate_left(8); // The internal model gets rotated somewhere on the loading. I
                                   // found out that spinning either by 8 positions (left for part,
                                   // right for int), works.

        for model in int_models {
            let local_vec: Vec<f32>;

            unsafe {
                let src = handler
                    .base_ptr()
                    .add((model.offset() + HEADER_SPACE) as usize)
                    as *const u8;
                let total_size = model
                    .resolve_header(handler.base_ptr())
                    .size
                    .load(Ordering::Relaxed);
                let payload_bytes = total_size - HEADER_SPACE;

                let byte_slice = std::slice::from_raw_parts(src, payload_bytes as usize);
                let raw_f32: &[f32] = bytemuck::cast_slice(&byte_slice);
                let expected_len = (FOURIER_RADIUS * FOURIER_RADIUS) as usize;
                local_vec = raw_f32[..expected_len.min(raw_f32.len())].to_vec();
            }

            cs_list.push(Self::biometric_distance(local_vec, part_model.clone()));
        }

        for cs in cs_list {
            println!("{:.20}", cs);
            if cs > 0.91 {
                println!("[FaceScape Auth] Welcome!");
                return;
            }
        }

        println!("[FaceScape Auth] Authentication failed")
    }

    /// Gets the specified user internal models from memory.
    ///
    /// This call will panic if the accessed matrix was not bootstrapped by an AuthManager.
    ///
    /// ### Params:
    /// @model_arena: A reference to the models MatrixHandler
    ///
    /// ### Returns:
    /// A list containing the pointers for each internal model
    fn get_int_models(model_arena: &MatrixHandler) -> Vec<RelativePtr<u8>> {
        let looper = Looper::new(model_arena.share());
        let mut init_flag = false;
        let mut ptr_list = Vec::<RelativePtr<u8>>::new();

        for w in looper {
            let state = w.view_header().state.load(Ordering::Acquire);

            if state == INIT_FLAG {
                init_flag = true;
            } else if state == RESIDENT_MODEL {
                ptr_list.push(RelativePtr::<u8>::new(w.view_offset()));
            } else {
                continue;
            }
        }

        if !init_flag {
            panic!("AuthManager is not initialized!");
        } else {
            return ptr_list;
        }
    }

    /// Generates a fresh facial model from the current challenger.
    ///
    /// The model is constructed from 30 captured frames.
    ///
    /// ### Returns:
    /// A vector of gray-scale frames in YUYV format.
    fn get_part_model() -> Vec<f32> {
        let webcam = WebcamIngress::new("/dev/video0", Some((1280, 720))).unwrap();

        let (w, h) = webcam.resolution();
        let fourier_engine = FourierFaceEngine::new(w as usize, h as usize);
        let temp_handler = AtomicMatrix::bootstrap(
            Some(format!("face_scape.auth.{}", uid_lite::generate_uuid())),
            memory_scale::custom::mb::<40>(),
        )
        .unwrap();

        let part_model = Constructor::run(&webcam, &fourier_engine, &temp_handler, 30);
        temp_handler.die();
        part_model
    }

    /// Calculates the cosine similarity between the two defined models in the struct.
    ///
    /// It first defines the dot product between the two models. Then it calculates the individual
    /// magnitude of each model, gets the square root of both magnitudes and divides the dot product
    /// by the magnitude multiplied from both models.
    ///
    /// ### Params:
    /// @int_model: The internal model of the user. \
    /// @part_model: The freshly collected model from the user.
    ///
    /// ### Returns:
    /// An f32 likeness score between the two models.
    fn biometric_distance(int_model: Vec<f32>, part_model: Vec<f32>) -> f32 {
        let dot_product: f32 = part_model
            .iter()
            .zip(int_model.iter())
            .map(|(a, b)| a * b)
            .sum();

        let mag_int: f32 = int_model.iter().copied().map(|x| x * x).sum();

        let mag_part: f32 = part_model.iter().copied().map(|x| x * x).sum();

        let mul_mag: f32 = mag_int * mag_part;
        let cosine_similarity: f32;

        if mul_mag > 0.0 {
            cosine_similarity = dot_product / mul_mag.sqrt();
        } else {
            cosine_similarity = 0.0;
        }

        cosine_similarity
    }
}
