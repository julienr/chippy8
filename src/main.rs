use eframe::{egui, egui_glow, glow};

use chippy8::array2d::Array2D;
use chippy8::machine::Machine;
use eframe::glow::HasContext;
use egui::mutex::Mutex;
use std::sync::Arc;

use chippy8::texture::Texture;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        multisampling: 4,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "Custom 3D painting in eframe using glow",
        options,
        Box::new(|cc| Box::new(MyApp::new(cc))),
    )
}

struct MyApp {
    /// Behind an `Arc<Mutex<…>>` so we can pass it to [`egui::PaintCallback`] and paint later.
    display_renderer: Arc<Mutex<DisplayRenderer>>,
    machine: Arc<Mutex<Machine>>,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc
            .gl
            .as_ref()
            .expect("You need to run eframe with the glow backend");
        let machine = Machine::default();
        Self {
            display_renderer: Arc::new(Mutex::new(DisplayRenderer::new(
                gl,
                machine.display.pixels.cols,
                machine.display.pixels.rows,
            ))),
            machine: Arc::new(Mutex::new(machine)),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("The triangle is being painted using ");
                ui.hyperlink_to("glow", "https://github.com/grovesNL/glow");
                ui.label(" (OpenGL).");
            });

            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                self.custom_painting(ui);
            });
            ui.label("Drag to rotate!");
        });
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(gl) = gl {
            self.display_renderer.lock().destroy(gl);
        }
    }
}

impl MyApp {
    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let display_renderer = self.display_renderer.clone();
        let pixels = self.machine.lock().display.pixels.clone();

        let (rect, _response) = ui.allocate_exact_size(
            egui::Vec2::new(pixels.cols as f32 * 10.0, pixels.rows as f32 * 10.0),
            egui::Sense::drag(),
        );

        let callback = egui::PaintCallback {
            rect,
            callback: std::sync::Arc::new(egui_glow::CallbackFn::new(move |_info, painter| {
                display_renderer.lock().paint(painter.gl(), &pixels);
            })),
        };
        ui.painter().add(callback);
    }
}

pub fn gl_error_to_string(err: u32) -> String {
    match err {
        glow::INVALID_ENUM => "GL_INVALID_ENUM".to_owned(),
        glow::INVALID_VALUE => "GL_INVALID_VALUE".to_owned(),
        glow::INVALID_OPERATION => "GL_INVALID_OPERATION".to_owned(),
        _ => {
            format!("Unhandled err: {:?}", err)
        }
    }
}

pub fn check_gl_errors_impl(gl: &glow::Context, file: &str, line: u32) {
    unsafe {
        loop {
            let err = gl.get_error();
            if err == glow::NO_ERROR {
                break;
            }
            println!(
                "OpenGL error: {:?} at {file}:{line}",
                gl_error_to_string(err)
            );
        }
    }
}

macro_rules! check_gl_errors {
    ($gl: expr) => {
        check_gl_errors_impl($gl, file!(), line!())
    };
}

struct DisplayRenderer {
    program: glow::Program,
    vertex_array: glow::VertexArray,
    texture: Texture,
}

impl DisplayRenderer {
    fn new(gl: &glow::Context, width: usize, height: usize) -> Self {
        use glow::HasContext as _;

        let shader_version = if cfg!(target_arch = "wasm32") {
            "#version 300 es"
        } else {
            "#version 330"
        };

        unsafe {
            let program = gl.create_program().expect("Cannot create program");

            let (vertex_shader_source, fragment_shader_source) = (
                r#"
                    const vec2 verts[4] = vec2[4](
                        vec2(-1.0, -1.0),
                        vec2(+1.0, -1.0),
                        vec2(-1.0, +1.0),
                        vec2(+1.0, +1.0)
                    );
                    const vec2 uvs[4] = vec2[4](
                        vec2(0.0, 0.0),
                        vec2(1.0, 0.0),
                        vec2(0.0, 1.0),
                        vec2(1.0, 1.0)
                    );
                    uniform float diffuse_aspect_ratio;
                    out vec2 v_uv;
                    void main() {
                        v_uv = uvs[gl_VertexID];
                        v_uv.y *= diffuse_aspect_ratio;
                        gl_Position = vec4(verts[gl_VertexID], 0.0, 1.0);
                    }
                "#,
                r#"
                    precision mediump float;
                    in vec2 v_uv;
                    out vec4 out_color;

                    uniform sampler2D diffuse;

                    void main() {
                        out_color = texture(diffuse, v_uv);
                    }
                "#,
            );

            let shader_sources = [
                (glow::VERTEX_SHADER, vertex_shader_source),
                (glow::FRAGMENT_SHADER, fragment_shader_source),
            ];

            let shaders: Vec<_> = shader_sources
                .iter()
                .map(|(shader_type, shader_source)| {
                    let shader = gl
                        .create_shader(*shader_type)
                        .expect("Cannot create shader");
                    gl.shader_source(shader, &format!("{shader_version}\n{shader_source}"));
                    gl.compile_shader(shader);
                    assert!(
                        gl.get_shader_compile_status(shader),
                        "Failed to compile {shader_type}: {}",
                        gl.get_shader_info_log(shader)
                    );
                    gl.attach_shader(program, shader);
                    shader
                })
                .collect();

            gl.link_program(program);
            assert!(
                gl.get_program_link_status(program),
                "{}",
                gl.get_program_info_log(program)
            );

            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            let vertex_array = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");

            let texture = Texture::checkerboard(gl, width, height);
            check_gl_errors!(gl);

            Self {
                program,
                vertex_array,
                texture,
            }
        }
    }

    fn destroy(&self, gl: &glow::Context) {
        use glow::HasContext as _;
        unsafe {
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vertex_array);
        }
    }

    fn paint(&self, gl: &glow::Context, pixels: &Array2D<bool>) {
        use glow::HasContext as _;
        self.texture.bind(gl, 0);
        unsafe {
            gl.use_program(Some(self.program));
            gl.uniform_1_i32(gl.get_uniform_location(self.program, "diffuse").as_ref(), 0);
            gl.uniform_1_f32(
                gl.get_uniform_location(self.program, "diffuse_aspect_ratio")
                    .as_ref(),
                pixels.rows as f32 / pixels.cols as f32,
            );
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        }
    }
}
