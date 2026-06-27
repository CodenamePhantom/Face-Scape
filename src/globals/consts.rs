/// Fourier signature matrix radius.
pub const FOURIER_RADIUS: u32 = 128;

/// .fmodel header versioniong
pub const FMODEL_MAGIC: &[u8] =  b"FSC\x01";

/// Internal model arena ID prefix.
pub const MODEL_ARENA: &'static str = &"face_scape.auth";

/// Init flag header state magic number. Used for AtomicMatrix coordination.
pub const INIT_FLAG: u32 = 1998;

/// Resident model header state magic number. Used for AtomicMatrix coordination.
pub const RESIDENT_MODEL: u32 = 1999;

/// OpenCV face crop image size.
pub const OPENCV_SCALE: u32 = 256;
