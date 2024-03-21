use std::process;

use windows_sys::Win32::Graphics::OpenGL::{
    glClear, glClearColor, glColorPointer, glCullFace, glDrawArrays, glEnableClientState,
    glFrontFace, glLoadIdentity, glMatrixMode, glVertexPointer, glViewport, gluLookAt,
    gluPerspective, SwapBuffers, GL_BACK, GL_CCW, GL_COLOR_ARRAY, GL_COLOR_BUFFER_BIT, GL_FLOAT,
    GL_MODELVIEW, GL_PROJECTION, GL_TRIANGLES, GL_VERTEX_ARRAY,
};
use xenocore as xc;

#[rustfmt::skip]
const VERTICES: &[[f32; 3]] = &[
    [0., 0., 0.],
    [1., 0., 0.],
    [0., 1., 0.],
];

#[rustfmt::skip]
const COLORS: &[[f32; 3]] = &[
    [1., 0., 0.],
    [0., 1., 0.],
    [0., 0., 1.],
];

fn main() {
    let window = xc::win32::Window::new(c"xenocore-gamoid");

    let mut vsync = unsafe { xc::wgl_load!(c"wglGetSwapIntervalEXT", fn() -> isize) }
        .map(|proc| unsafe { proc() } != 0)
        .unwrap_or_default();

    unsafe {
        glClearColor(0., 0., 0., 1.);
        glEnableClientState(GL_VERTEX_ARRAY);
        glEnableClientState(GL_COLOR_ARRAY);
        glFrontFace(GL_CCW);
        glCullFace(GL_BACK);
    };

    window.event_loop(|event| {
        match event {
            xc::Event::Press(xc::key::ESCAPE | xc::key::Q) => {
                process::exit(0);
            }

            xc::Event::Press(xc::key::V) => {
                unsafe { toggle_vsync(&mut vsync) };
                return;
            }

            _ => {}
        }

        let [width, height] = window.inner_size();
        let aspect = width as f64 / height as f64;

        unsafe {
            glMatrixMode(GL_PROJECTION);
            glLoadIdentity();
            gluPerspective(90., aspect, 1e-1, 1e3);

            glMatrixMode(GL_MODELVIEW);
            glLoadIdentity();
            gluLookAt(0., 0., 3., 0., 0., 0., 0., 1., 0.);

            glViewport(0, 0, width as _, height as _);
            glClear(GL_COLOR_BUFFER_BIT);

            assert_eq!(VERTICES.len(), COLORS.len());
            glVertexPointer(3, GL_FLOAT, 0, VERTICES.as_ptr() as _);
            glColorPointer(3, GL_FLOAT, 0, COLORS.as_ptr() as _);
            glDrawArrays(GL_TRIANGLES, 0, 3);

            SwapBuffers(window.hdc);
        }
    });
}

unsafe fn toggle_vsync(vsync: &mut bool) {
    *vsync ^= true;

    #[allow(non_snake_case)]
    if let Some(wglSwapIntervalEXT) = xc::wgl_load!(c"wglSwapIntervalEXT", fn(isize) -> isize) {
        wglSwapIntervalEXT(*vsync as _);
    }
}
