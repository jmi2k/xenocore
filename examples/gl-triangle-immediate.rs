use windows_sys::Win32::Graphics::OpenGL::{
    glBegin, glClear, glClearColor, glColor3f, glEnd, glVertex3f, glViewport, wglCreateContext,
    wglDeleteContext, wglMakeCurrent, SwapBuffers, GL_COLOR_BUFFER_BIT, GL_TRIANGLES,
};
use xenocore as xc;

fn main() {
    let window = xc::win32::Window::new(c"xenocore-gl-triangle-immediate");

    let hglrc = unsafe { wglCreateContext(window.hdc) };
    assert!(hglrc != 0);

    unsafe {
        glClearColor(0., 0., 0., 1.);
    };

    window.event_loop(|event| {
        if let xc::Event::Press(_) = event {
            return;
        }

        let [width, height] = window.inner_size();

        unsafe {
            glViewport(0, 0, width as _, height as _);
            glClear(GL_COLOR_BUFFER_BIT);

            glBegin(GL_TRIANGLES);

            glVertex3f(0., 0., 0.);
            glColor3f(1., 0., 0.);
            glVertex3f(1., 0., 0.);
            glColor3f(0., 1., 0.);
            glVertex3f(0., 1., 0.);
            glColor3f(0., 0., 1.);

            glEnd();

            SwapBuffers(window.hdc);
        }
    });

    unsafe {
        wglMakeCurrent(0, 0);
        wglDeleteContext(hglrc);
    }
}
