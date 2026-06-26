use opencv::{
    core::{ Mat, Rect, Size },
    objdetect::CascadeClassifier,
    prelude::*,
    imgproc::{ resize, INTER_AREA }
};
use std::path::Path;

pub struct FacialDetector {
    cascade: CascadeClassifier,
    target_size: i32,
}

impl FacialDetector {
    pub fn new<P: AsRef<Path>>(xml_path: P, target_size: i32) -> Result<Self, opencv::Error> {
        let path_str = xml_path.as_ref().to_string_lossy();
        let cascade = CascadeClassifier::new(&path_str)?;

        Ok(Self {
            cascade,
            target_size,
        })
    }

    pub fn crop_and_normalize_face(&mut self, frame: &Mat) -> Result<Option<Mat>, opencv::Error> {
        let mut faces = opencv::core::Vector::<Rect>::new();

        self.cascade.detect_multi_scale(
            &frame,
            &mut faces,
            1.1,
            5,
            0,
            Size::new(60, 60),
            Size::new(0, 0)
        )?;

        if faces.is_empty() {
            return Ok(None);
        }

        let mut best_face = faces.get(0)?;
        for i in 1..faces.len() {
            let current_face = faces.get(i)?;
            if (current_face.width * current_face.height) > (best_face.width * best_face.height) {
                best_face = current_face;
            }
        }

        let cropped_face = Mat::roi(frame, best_face)?;
            
        let mut normalized_face = Mat::default();
        let final_size = Size::new(self.target_size, self.target_size);

        resize(&cropped_face, &mut normalized_face, final_size, 0.0, 0.0, opencv::imgproc::INTER_CUBIC)?;

        Ok(Some(normalized_face))
    }
}
