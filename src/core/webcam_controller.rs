use std::io;
use v4l::buffer::Type;
use v4l::io::traits::CaptureStream;
use v4l::video::Capture;
use v4l::{Device, FourCC};

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
            None => (1024, 1980) // Defaults to FullHD
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

        for _ in 0..frame_capture {
            let (buffer, _) = stream.next()?;

            let total_pixels = (self.width * self.height) as usize;
            let mut gray_buffer = Vec::with_capacity(total_pixels);

            for chunk in buffer.chunks_exact(2) {
                gray_buffer.push(chunk[0]);
            }

            frame_pack.push(Frames { slice: gray_buffer });
        }

        Ok(frame_pack)
    }

    pub fn resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}
