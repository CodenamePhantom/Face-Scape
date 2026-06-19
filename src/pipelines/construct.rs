use crate::core::fourier_engine::FourierFaceEngine;
use crate::core::webcam_controller::WebcamIngress;

pub struct Constructor<'a> {
    fourier_engine: &'a FourierFaceEngine,
    webcam: &'a WebcamIngress,
}

impl<'a> Constructor<'a> {
    pub fn run(
        webcam: &'a WebcamIngress,
        f_engine: &'a FourierFaceEngine,
    ) ->Vec<f32> {
        let constructor = Self {
            fourier_engine: f_engine,
            webcam,
        };

        let f_frame_pack = constructor.capture_frames(30);
        let centroid = FourierFaceEngine::centroid_frame_generator(f_frame_pack);

        centroid
    }

    fn capture_frames(&self, frames: u8) -> Vec<Vec<f32>> {
        let gs_frames = self.webcam.capture_gray_scale_frames(frames).unwrap();

        let mut f_frame_pack = Vec::new();

        for frame in gs_frames {
            let f_frame = self.fourier_engine.process_frame_to_coefficients(&frame.slice);
            f_frame_pack.push(f_frame)
        }

        f_frame_pack
    }
}
