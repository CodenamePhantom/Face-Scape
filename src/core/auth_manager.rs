use crate::globals::consts::{INIT_FLAG, MODEL_ARENA, RESIDENT_MODEL};
use crate::model::face_distance_model;
use crate::pipelines::enroll::Enroll;
use atomic_matrix::extensive_lib::looper::Looper;
use atomic_matrix::prelude::{
    AtomicMatrix, HEADER_SPACE, HandlerFunctions, MatrixHandler, STATE_FREE, memory_scale,
};
use std::sync::atomic::Ordering;

pub struct AuthManager {
    _handler: MatrixHandler,
}

impl AuthManager {
    pub fn start(user: String) {
        let pid = std::process::id();
        let handler = AtomicMatrix::bootstrap(
            Some(format!("{}.{}", MODEL_ARENA, user)),
            memory_scale::custom::mb::<5>(),
        )
        .unwrap();
        let init_flag = handler.allocate_raw(36).unwrap(); // Alloc collapses to HEADER_SPACE in the
        // matrix by default

        unsafe {
            init_flag
                .resolve_header(handler.base_ptr())
                .state
                .store(INIT_FLAG, Ordering::Release);

            let body_ptr = handler.base_ptr().add(init_flag.offset() as usize) as *mut u32;
            std::ptr::write(body_ptr, pid);
        }

        let models = face_distance_model::parse(&user);

        for model in models {
            let payload: &[f32] = bytemuck::cast_slice(&model);
            let size = payload.len();
            let byte_len = std::mem::size_of_val(payload);

            let rel_ptr = handler.allocate_raw(byte_len as u32).unwrap();

            unsafe {
                rel_ptr
                    .resolve_header(handler.base_ptr())
                    .state
                    .store(RESIDENT_MODEL, Ordering::Release);

                let dst = handler.base_ptr().add(rel_ptr.offset() as usize) as *mut f32;
                std::ptr::copy_nonoverlapping(payload.as_ptr(), dst, size);
            }
        }

        let _auth_manager = Self { _handler: handler };

        println!("[FaceScape - AuthManager] Models preloaded in shared memory.");

        loop {
            std::thread::sleep(std::time::Duration::from_secs(60));
        }
    }

    pub fn stop(user: String) {
        let handler = AtomicMatrix::bootstrap(
            Some(format!("{}.{}", MODEL_ARENA, user)),
            memory_scale::custom::mb::<15>(),
        )
        .unwrap();
        let looper = Looper::new(handler.share());

        let mut closed = false;

        for w in looper {
            if w.view_header().state.load(Ordering::Acquire) == INIT_FLAG {
                unsafe {
                    let pid = handler.base_ptr().add(w.view_offset() as usize) as *const u32;
                    libc::kill(pid as i32, libc::SIGKILL);
                }

                handler.free_at(w.view_offset() - HEADER_SPACE);

                closed = true;
            } else if w.view_header().state.load(Ordering::Acquire) != STATE_FREE {
                handler.free_at(w.view_offset() - HEADER_SPACE);
            }
        }

        handler.die();

        if closed {
            println!("[FaceScape] AuthManager stopped successfully.");
        } else {
            println!("[FaceScape] Failed to close AuthManager! No initialization PID found");
        }
    }

    pub fn list() {
        let f_iter = std::fs::read_dir("/etc/facescape/").unwrap();

        println!(" Idx    | Name             | Modified");
        for (i, fmodel) in f_iter.enumerate() {
            let model = fmodel.unwrap();

            println!("--------------------------");
            println!(
                "{:4}    | {:?}    | {:?}",
                i,
                model.file_name(),
                model.metadata().unwrap().modified().unwrap()
            );
            println!("--------------------------");
        }
    }

    pub fn delete(user: String) {
        std::fs::remove_file(format!("/etc/facescape/{}.fmodel", user)).unwrap();
        Self::stop(user);
    }

    pub fn update(user: String) {
        Self::delete(user.clone());
        Enroll::enroll(user);
    }

    pub fn reload(user: String) {
        Self::stop(user.clone());
        Self::start(user);
    }
}
