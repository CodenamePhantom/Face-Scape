use crate::globals::gamma_lut;
use crate::model::cascade_classifier::FacialDetector;
use std::io;
use v4l::buffer::Type;
use v4l::io::traits::CaptureStream;
use v4l::video::Capture;
use v4l::{Device, FourCC};
use opencv::{
    core::Mat,
    prelude::*,
};

const FIRST_FRAMES_DROP: u8 = 7; // we drop the v4l gamma correction window.

pub struct WebcamIngress {
    device: Device,
    width: u32,
    height: u32,
}

pub struct Frames {
    pub slice: Vec<u8>,
}

impl WebcamIngress {
    pub fn new(path: &str, targets: Option<(u32, u32)>) -> io::Result<Self> {
        let device = Device::with_path(path)?;
        let (width, height) = match targets {
            Some(v) => v,
            None => (1920, 1024) // Defaults to FullHD
        };

        let mut format = device.format()?;
        format.width = width;
        format.height = height;
        format.fourcc = FourCC::new(b"YUYV");

        let format = device.set_format(&format)?;

        println!(
            "[Webcam] Hardware initialized: {}x{} [FourCC: {}]",
            format.width, format.height, format.fourcc
        );

        Ok(Self {
            device,
            width: format.width,
            height: format.height,
        })
    }

    pub fn capture_gray_scale_frames(&self, frame_capture: u8) -> io::Result<Vec<Frames>> {
        let mut stream = v4l::io::mmap::Stream::new(&self.device, Type::VideoCapture)?;
        let mut frame_pack = Vec::<Frames>::with_capacity(frame_capture as usize);

        for i in 0..frame_capture + FIRST_FRAMES_DROP {
            let (buffer, _) = stream.next()?;

            if i < FIRST_FRAMES_DROP {
                continue
            };

            let total_pixels = (self.width * self.height) as usize;
            let mut gray_buffer = Vec::with_capacity(total_pixels);

            for chunk in buffer.chunks_exact(2) {
                gray_buffer.push(chunk[0]);
            }

            frame_pack.push(Frames { slice: gray_buffer });
        }

        Ok(frame_pack)
    }

    pub fn gamma_crushed_frames(&self, frame_capture: u8) -> io::Result<Vec<Frames>> {
        let mut stream = v4l::io::mmap::Stream::new(&self.device, Type::VideoCapture)?;
        let mut frame_pack = Vec::<Frames>::with_capacity(frame_capture as usize);

        let gamma_lut = gamma_lut::generate_gamma_lut(0.40, 1.6, -20.0);

        for i in 0..frame_capture + FIRST_FRAMES_DROP {
            let (buffer, _) = stream.next()?;

            if i < FIRST_FRAMES_DROP {
                continue
            };

            let total_pixels = (self.width * self.height) as usize;
            let mut gray_buffer = Vec::with_capacity(total_pixels);

            for chunk in buffer.chunks_exact(2)  {
                let crushed_y = gamma_lut[chunk[0] as usize] as u8;
                gray_buffer.push(crushed_y);
            }

            frame_pack.push(Frames { slice: gray_buffer });
        }

        Ok(frame_pack)
    }

    pub fn face_crop(&self, frame: &[u8], f_detector: &mut FacialDetector) -> Frames {
        let borrowed_mat = Mat::new_rows_cols_with_data( 
            self.height as i32, 
            self.width as i32, 
            frame,
        ).unwrap();
        let mat = borrowed_mat.try_clone().unwrap();

        if let Some(face_mat) = f_detector.crop_and_normalize_face(&mat).unwrap() {
            opencv::imgcodecs::imwrite("assets/face_debug.png", &face_mat, &opencv::core::Vector::new()).unwrap();

            let continuous_face = if face_mat.is_continuous() {
                face_mat
            } else {
                face_mat.clone()
            };

            let face_bytes_slice = continuous_face.data_bytes().unwrap();
            let face_vector_owned = face_bytes_slice.to_vec();

            return Frames { slice: face_vector_owned }
        } else {
            panic!("Unable to capture facial frames!")
        }
    }

    pub fn resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}
