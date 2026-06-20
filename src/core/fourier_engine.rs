use crate::globals::consts::*;
use ndarray::Array2;
use rustfft::{FftPlanner, num_complex::Complex};
use std::sync::Arc;

/// FourierFaceEngine stores all the data and methods necessary to convert webcam frames into
/// normalized fourier matrices while preserving critical information from the face to make
/// authentication feasible.
pub struct FourierFaceEngine {
    width: usize,
    height: usize,
    fft_width: Arc<dyn rustfft::Fft<f32>>,
    fft_height: Arc<dyn rustfft::Fft<f32>>,
}

impl FourierFaceEngine {
    /// Creates a new FourierFaceEngine instance and returns it to the user.
    ///
    /// ### Params:
    /// @width: The width of the Fourier transformed matrix. \
    /// @height: The height of the Fourier transformed matrix.
    ///
    /// ### Returns:
    /// An instance of Self.
    pub fn new(width: usize, height: usize) -> Self {
        let mut planner = FftPlanner::new();

        let fft_width = planner.plan_fft_forward(width);
        let fft_height = planner.plan_fft_forward(height);

        Self {
            width,
            height,
            fft_width,
            fft_height,
        }
    }

    /// Applies a frequency normalization filter on a frame to return a transformed vector of f32
    /// weights.
    ///
    /// The frame first passes through a Sobel filter to extract the shadow map based on the color
    /// gradient decay of neighbours. Then a 2D Fast Fourier Transformation is applied to both rows
    /// and columns to separate the frequency identities on the shadow map, which is then run
    /// through A Difference of Gaussians mask to filter out high and low frequency noise unusable
    /// to the facial map.
    ///
    /// The output frame is then extracted into a 48 x 48 radius fourier signature, and passed
    /// through an L2 Block Filter to even the weights between 0.0 and 1.0.
    ///
    /// ### Params:
    /// @gray_frame: The frame in YUYV format as a stream of bytes.
    ///
    /// ### Returns:
    /// The normalized fourier frame of the original frame.
    pub fn process_frame_to_coefficients(&self, gray_frame: &[u8]) -> Vec<f32> {
        let shadow_matrix = self.sobel_shadow_map(gray_frame);

        // Applies a 2D FFT transformation to the shadow map. First the rows, then the columns
        let mut complex_matrix = self.fast_fourier_transform(shadow_matrix);

        // Applies a DoG (Difference of Gaussians) blur to the transformed matrix to normalize high
        // frequency noise and exclude background information, dust, webcam artifacts, and a few other
        // small glitches.
        complex_matrix = self.difference_of_gaussians(complex_matrix);

        // Runs the fourier signature to extract a determined radius of information from the complex
        // matrix into f32 format.
        let scan_radius = FOURIER_RADIUS as usize;
        let mut fourier_signature = Vec::with_capacity(scan_radius * scan_radius);

        for y in 0..scan_radius {
            for x in 0..scan_radius {
                if y < 4 && x < 4 {
                    fourier_signature.push(0.0)
                } else {
                    let magnitude = complex_matrix[[y, x]].norm();
                    fourier_signature.push(magnitude);
                }
            }
        }

        // L2 Block Filter pass.
        let norm: f32 = fourier_signature.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-10 {
            fourier_signature.iter_mut().for_each(|x| *x /= norm);
        }

        fourier_signature
    }

    // Aggregate a given number of fourier_frames using the median of all frames to assemble a centroid.
    //
    // ### Params:
    // @fourier_frames: A package containing N normalized fourier_frames.
    pub fn centroid_frame_generator(fourier_frames: Vec<Vec<f32>>) -> Vec<f32> {
        if fourier_frames.is_empty() {
            return Vec::new();
        }

        let num_frames = fourier_frames.len();
        let num_pixels = fourier_frames[0].len();

        let mut centroid = vec![0.0f32; num_pixels];

        let mut pixel_timeline = [0.0f32; 32];
        let num_frames_safe = num_frames.min(32);
        let median_idx = num_frames_safe / 2;

        for pixel_idx in 0..num_pixels {
            for t in 0..num_frames_safe {
                unsafe {
                    pixel_timeline[t] = *fourier_frames.get_unchecked(t).get_unchecked(pixel_idx);
                }
            }

            let active_slice = &mut pixel_timeline[0..num_frames_safe];
            active_slice
                .sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            centroid[pixel_idx] = active_slice[median_idx];
        }

        centroid
    }

    /// Applies a Sobel Filter algorithm to a given frame, in order to extract a shadow_matrix based
    /// on the gradient B&W decay of neighbours.
    ///
    /// ### Params:
    /// @grey_frame: The frame in YUYV format as a stream of bytes.
    ///
    /// ### Returns:
    /// The 2D shadow matrix.
    fn sobel_shadow_map(
        &self,
        gray_frame: &[u8],
    ) -> ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>, f32> {
        let img_matrix = Array2::from_shape_vec((self.height, self.width), gray_frame.to_vec())
            .unwrap_or_else(|_| Array2::zeros((self.height, self.width)));

        let mut shadow_matrix = Array2::<f32>::zeros((self.height, self.width));
        for row in 1..(self.height - 1) {
            for col in 1..(self.width - 1) {
                let dx = img_matrix[[row, col + 1]] as f32 - img_matrix[[row, col - 1]] as f32;
                let dy = img_matrix[[row + 1, col]] as f32 - img_matrix[[row - 1, col]] as f32;

                shadow_matrix[[row, col]] = (dx * dx + dy * dy).sqrt();
            }
        }

        shadow_matrix
    }

    /// Runs a 2D Fast Fourier Transformation inside a given matrix to separate the noise
    /// frequencies of the mathematical representation of the image.
    ///
    /// First it process each row sequentially, then moves to processing columns. All cells inside
    /// the matrix are loaded into a static vec to avoid heap allocations and increase processing
    /// performance.
    ///
    /// ### Params:
    /// @shadow_matrix: The matrix representing the calculated shadow map of the frame.
    ///
    /// ### Returns:
    /// A complex matrix representing the transformed floating point values.
    fn fast_fourier_transform(
        &self,
        shadow_matrix: ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>, f32>,
    ) -> ndarray::ArrayBase<ndarray::OwnedRepr<Complex<f32>>, ndarray::Dim<[usize; 2]>, Complex<f32>>
    {
        let mut complex_matrix = shadow_matrix.mapv(|val| Complex::new(val, 0.0));

        let row_scratch_len = self.fft_width.get_inplace_scratch_len().max(self.width);
        let mut row_scratch = vec![Complex::new(0.0, 0.0); row_scratch_len];
        let mut row_buffer = vec![Complex::new(0.0, 0.0); self.width];

        for row in 0..self.height {
            for col in 0..self.width {
                row_buffer[col] = complex_matrix[[row, col]];
            }

            self.fft_width
                .process_with_scratch(&mut row_buffer, &mut row_scratch);

            for col in 0..self.width {
                complex_matrix[[row, col]] = row_buffer[col];
            }
        }

        let col_scratch_len = self.fft_height.get_inplace_scratch_len().max(self.height);
        let mut col_scratch = vec![Complex::new(0.0, 0.0); col_scratch_len];
        let mut col_buffer = vec![Complex::new(0.0, 0.0); self.height];

        for col in 0..self.width {
            for row in 0..self.height {
                col_buffer[row] = complex_matrix[[row, col]];
            }

            self.fft_height
                .process_with_scratch(&mut col_buffer, &mut col_scratch);

            for row in 0..self.height {
                complex_matrix[[row, col]] = col_buffer[row];
            }
        }

        complex_matrix
    }

    /// Applies a Difference of Gaussians mask into a fourier frame to aggregate mid to high level
    /// frequencies while also working as a band pass filter for low and ultra-high frequency noise.
    ///
    /// DoG was choosen in this case, as it works best for Edge Detection and fine details
    /// enhancement while still removing noise from the matrix, as opposed to common Gaussian
    /// Blurring that only removes low frequency noise.
    ///
    /// ### Params:
    /// @complex_matrix: The fourier frame matrix to be filtered.
    ///
    /// ### Returns:
    /// The filtered complex matrix.
    pub fn difference_of_gaussians(
        &self,
        mut complex_matrix: ndarray::ArrayBase<
            ndarray::OwnedRepr<Complex<f32>>,
            ndarray::Dim<[usize; 2]>,
            Complex<f32>,
        >,
    ) -> ndarray::ArrayBase<ndarray::OwnedRepr<Complex<f32>>, ndarray::Dim<[usize; 2]>, Complex<f32>>
    {
        let sigma_small: f32 = 4.0 / self.width as f32;
        let sigma_large: f32 = 18.0 / self.width as f32;
        let s2_small_inv = -1.0 / (2.0 * sigma_small * sigma_small);
        let s2_large_inv = -1.0 / (2.0 * sigma_large * sigma_large);

        let w_f32 = self.width as f32;
        let h_f32 = self.height as f32;

        for row in 0..self.height {
            let fy = if row < self.height / 2 {
                row as f32
            } else {
                row as f32 - h_f32
            } / h_f32;
            let fy2 = fy * fy;

            for col in 0..self.width {
                let fx = if col < self.width / 2 {
                    col as f32
                } else {
                    col as f32 - w_f32
                } / w_f32;
                let d2 = fx * fx + fy2;

                let g_small = (d2 * s2_small_inv).exp();
                let g_large = (d2 * s2_large_inv).exp();
                let dog_weight = g_small - g_large;

                complex_matrix[[row, col]] *= dog_weight;
            }
        }

        complex_matrix
    }
}
