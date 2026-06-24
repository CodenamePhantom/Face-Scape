use crate::{core::fourier_engine::FourierFaceEngine, globals::consts::FOURIER_RADIUS};
use crate::core::webcam_controller::WebcamIngress;
use atomic_matrix::prelude::{
    HEADER_SPACE, HandlerFunctions, MatrixHandler, RelativePtr
};
use std::{sync::atomic::Ordering, thread::ScopedJoinHandle};

const STATE_PROCESSING: u32 = 50;

/// Constructor holds the pipeline for assemblying facial models
///
/// Constructor invariantly requires access to an AtomicMatrix instance throug MatrixHandler for
/// constructing fourier frames in parallel.
///
/// The pipeline is:
///
///    [capture frames (30 frames)]
///                 |
///     |-----------|-----------|-...
///     |           |           |
///  [FFT 1]     [FFT 2]     [FFT 3]
///     |           |           |
///     |-----------------------|
///                 |
///        [Median aggregation]
///                 |
///         [return centroid]
pub struct Constructor<'a> {
    fourier_engine: &'a FourierFaceEngine,
    webcam: &'a WebcamIngress,
}

impl<'a> Constructor<'a> {
    /// Runs the construction pipeline.
    ///
    /// ### Params:
    /// @webcam: An instance of WebcamIngress to capture frames. \
    /// @f_engine: A FourierFaceEngine instance to process the frames. \
    /// @handler: MatrixHandler instance for parallel processing.
    ///
    /// ### Returns:
    /// The constructed centroid.
    pub fn run(
        webcam: &'a WebcamIngress,
        f_engine: &'a FourierFaceEngine,
        handler: &'a MatrixHandler,
    ) -> Vec<f32> {
        let constructor = Self {
            fourier_engine: f_engine,
            webcam,
        };

        let blocks = constructor.capture_frames(30, handler);
        let f_blocks = constructor.generate_fourier_frames(blocks, &handler);
        let centroid = constructor.construct_centroid(f_blocks, &handler);

        centroid
    }

    /// Captures N frames from WebcamIngress and dump them into the AtomicMatrix.
    ///
    /// ### Params:
    /// @frames: The quantity of frames to be captured. \
    /// @handler: A reference to the MatrixHandler.
    ///
    /// ### Returns:
    /// A list of RelativePtr for the frames inside the arena.
    fn capture_frames(&self, frames: u8, handler: &MatrixHandler) -> Vec<RelativePtr<u8>> {
        let gs_frames = self.webcam.capture_gray_scale_frames(frames).unwrap();

        let mut f_frame_pack = Vec::new();

        for frame in gs_frames {
            let payload: &[u8] = bytemuck::cast_slice(&frame.slice);
            let size = payload.len();
            let rel_ptr = handler.allocate_raw(size as u32).unwrap();

            unsafe {
                let dst = handler.base_ptr().add(rel_ptr.offset() as usize) as *mut u8;
                std::ptr::copy_nonoverlapping(payload.as_ptr(), dst, size);
            }
            f_frame_pack.push(rel_ptr);
        }

        return f_frame_pack;
    }

    /// Processes the captured frames inside AtomicMatrix into transformed fourier frames.
    ///
    /// It spawns worker threads with chunks of 6 pointers per thread and a reference to the
    /// MatrixHandler. These workers then execute the job of generating the fourier frames from the
    /// original frames inside the AtomicMatrix, and publish the processed version again in the
    /// arena, all in parallel.
    ///
    /// ### Params:
    /// @blocks: The list of RelativePtr to process.
    /// @handler: A reference to MatrixHandler to be passed between threads.
    ///
    /// ### Returns:
    /// A list of RelativePtr for the fourier frames inside the arena.
    fn generate_fourier_frames(
        &self,
        blocks: Vec<RelativePtr<u8>>,
        handler: &MatrixHandler,
    ) -> Vec<RelativePtr<u8>> {
        let mut f_blocks = Vec::with_capacity(blocks.len());

        fn job(
            handler_inner: &MatrixHandler,
            block_chunk: &[RelativePtr<u8>],
            f_engine: &FourierFaceEngine,
        ) -> Vec<RelativePtr<u8>> {
            let mut p_block = Vec::new();
            for block in block_chunk {
                unsafe {
                    block
                        .resolve_header(handler_inner.base_ptr())
                        .state
                        .store(
                            STATE_PROCESSING,
                            Ordering::Release,
                        );
                };

                let local_vec: Vec<u8>;
                unsafe {
                    let src = handler_inner.base_ptr().add(block.offset() as usize) as *const u8;
                    let total_size = block
                        .resolve_header(handler_inner.base_ptr())
                        .size
                        .load(Ordering::Relaxed);
                    let payload_bytes = total_size - HEADER_SPACE;

                    let byte_slice = std::slice::from_raw_parts(src, payload_bytes as usize);
                    local_vec = byte_slice.to_vec();
                }
                handler_inner.free_at(block.offset() - HEADER_SPACE);

                let fourier_frame = f_engine.process_frame_to_coefficients(&local_vec);

                let payload: &[f32] = bytemuck::cast_slice(&fourier_frame);
                let size = payload.len();
                let byte_len = std::mem::size_of_val(payload);

                let rel_ptr = handler_inner.allocate_raw(byte_len as u32).unwrap();

                unsafe {
                    let dst = handler_inner.base_ptr().add(rel_ptr.offset() as usize) as *mut f32;
                    std::ptr::copy_nonoverlapping(payload.as_ptr(), dst, size);
                }
                p_block.push(rel_ptr)
            }

            return p_block;
        }

        std::thread::scope(|s| {
            let handler_ref = handler;
            let f_engine = self.fourier_engine;

            let mut handles = Vec::<ScopedJoinHandle<Vec<RelativePtr<u8>>>>::new();

            for block_scope in blocks.chunks(6) {
                handles.push(s.spawn(move || job(handler_ref, block_scope, f_engine)));
            }

            for handle in handles {
                f_blocks.extend(handle.join().unwrap())
            }
        });

        return f_blocks;
    }

    /// Construct a median centroid frame from the fourier frames inside the matrix.
    ///
    /// Frames are queried sequentially from the arena and aggregated through the median of all
    /// values.
    ///
    /// ### Params:
    /// @blocks: The list of RelativePtr to process.
    /// @handler: A reference to MatrixHandler.
    ///
    /// ### Returns:
    /// The constructed centroid.
    fn construct_centroid(
        &self,
        blocks: Vec<RelativePtr<u8>>,
        handler: &MatrixHandler,
    ) -> Vec<f32> {
        let mut f_frames = Vec::<Vec<f32>>::new();

        for block in blocks {
            let local_vec: Vec<f32>;
            unsafe {
                let src = handler.base_ptr().add(block.offset() as usize) as *const u8;
                let total_size = block
                    .resolve_header(handler.base_ptr())
                    .size
                    .load(Ordering::Relaxed);

                let payload_bytes = total_size - HEADER_SPACE;

                let raw_bytes = std::slice::from_raw_parts(src, payload_bytes as usize);

                let byte_slice: &[f32] = bytemuck::cast_slice(raw_bytes);
                let expected_len = (FOURIER_RADIUS * FOURIER_RADIUS) as usize;
                let trimmed = &byte_slice[..expected_len.min(byte_slice.len())];
                local_vec = trimmed.to_vec();
            }
            handler.free_at(block.offset() - HEADER_SPACE);

            f_frames.push(local_vec);
        }

        let centroid = FourierFaceEngine::centroid_frame_generator(f_frames);
        centroid
    }
}
