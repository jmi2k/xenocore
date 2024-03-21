use windows_sys::Win32::Graphics::OpenGL::{
    glClear, glClearColor, glColorPointer, glDrawArrays, glEnableClientState, glVertexPointer,
    glViewport, SwapBuffers, GL_COLOR_ARRAY, GL_COLOR_BUFFER_BIT, GL_FLOAT, GL_TRIANGLES,
    GL_VERTEX_ARRAY,
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
    let window = xc::win32::Window::new(c"xenocore-gl-triangle-arrays");

    unsafe {
        glClearColor(0., 0., 0., 1.);
        glEnableClientState(GL_VERTEX_ARRAY);
        glEnableClientState(GL_COLOR_ARRAY);
    };

    window.event_loop(|event| {
        if let xc::Event::Press(_) = event {
            return;
        }

        let [width, height] = window.inner_size();

        unsafe {
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
