use opencv::{
    core::{ Mat, Rect, Size },
    objdetect::CascadeClassifier,
    prelude::*,
    imgproc::resize,
};
use std::path::Path;

/// Uses OpenCV Cascade Classifying specifications to detect and crop faces on frames.
pub struct FacialDetector {
    cascade: CascadeClassifier,
    target_size: i32,
}

impl FacialDetector {
    /// Spaws a new FacialDetector instance from the haar cascade xml and a target crop size.
    ///
    /// ### Params:
    /// @xml_path: The path to the Haar Cascade xml. \
    /// @target_size: The size on which the crop will be sampled.
    ///
    /// ### Returns:
    /// A result containing either an instance of self, or an error.
    pub fn new<P: AsRef<Path>>(xml_path: P, target_size: i32) -> Result<Self, opencv::Error> {
        let path_str = xml_path.as_ref().to_string_lossy();
        let cascade = CascadeClassifier::new(&path_str)?;

        Ok(Self {
            cascade,
            target_size,
        })
    }

    /// Runs the frame against the cascade heuristics to detect and crop the face.
    ///
    /// ### Params:
    /// @frame: The frame to crop the face from, in Mat format.
    ///
    /// ### Returns:
    /// A result containing either an optional Mat (None is returned if no faces are found.), or an
    /// error.
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
