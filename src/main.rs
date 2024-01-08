use eframe::egui::InputState;
use eframe::emath::Align;
use eframe::{egui, egui_glow, glow};
use std::time::Duration;

use chippy8::machine::Machine;
use chippy8::texture::RGBAImage;
use eframe::glow::HasContext;
use egui::mutex::Mutex;
use egui_extras::{Column, TableBuilder};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use chippy8::texture::Texture;

const TARGET_INSTRUCTIONS_PER_SECOND: u32 = 700;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        multisampling: 4,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native("Chippy8", options, Box::new(|cc| Box::new(MyApp::new(cc))))
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum ExecutionMode {
    StepByStep,
    Continuous,
}

#[derive(Debug)]
enum Message {
    ChangeMode(ExecutionMode),
    ExecuteOne,
    Exit,
}

fn machine_thread(machine: Arc<Mutex<Machine>>, rx: Receiver<Message>) {
    let mut execution_mode = ExecutionMode::StepByStep;
    let sleep_duration = Duration::new(0, 10e9 as u32 / TARGET_INSTRUCTIONS_PER_SECOND);
    loop {
        // Handle messages if any
        let msg = rx.try_recv();
        match msg {
            Ok(Message::ExecuteOne) => {
                machine.lock().execute_one();
            }
            Ok(Message::ChangeMode(mode)) => {
                execution_mode = mode;
            }
            Ok(Message::Exit) => break,
            Err(_) => {}
        }
        // Depending on execution mode, either do next instruction
        // or do nothing (if step by step)
        if execution_mode == ExecutionMode::Continuous {
            machine.lock().execute_one();
        }
        // Sleep to aim for target instructions per second
        thread::sleep(sleep_duration);
    }
}

struct MyApp {
    /// Behind an `Arc<Mutex<…>>` so we can pass it to [`egui::PaintCallback`] and paint later.
    display_renderer: Arc<Mutex<DisplayRenderer>>,
    machine: Arc<Mutex<Machine>>,
    follow_pc: bool,
    machine_thread_handle: Option<JoinHandle<()>>,
    machine_thread_tx: Sender<Message>,
    execution_mode: ExecutionMode,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc
            .gl
            .as_ref()
            .expect("You need to run eframe with the glow backend");
        let mut machine = Machine::default();
        let display_width = machine.display.width();
        let display_height = machine.display.height();
        machine.load_rom_from_file("roms/ibm_logo.ch8").unwrap();
        // machine.load_rom_from_file("roms/test_opcode.ch8").unwrap();
        // TODO: Display rom in debug ui
        let machine = Arc::new(Mutex::new(machine));

        let (tx, rx) = channel::<Message>();
        let machine_clone = machine.clone();
        let handle = thread::spawn(move || machine_thread(machine_clone, rx));

        Self {
            display_renderer: Arc::new(Mutex::new(DisplayRenderer::new(
                gl,
                display_width,
                display_height,
            ))),
            machine,
            follow_pc: true,
            machine_thread_handle: Some(handle),
            machine_thread_tx: tx,
            execution_mode: ExecutionMode::StepByStep,
        }
    }

    fn play_rom(&mut self, filepath: &str) {
        println!("Loading file {}", filepath);
        *self.machine.lock() = Machine::default();
        self.machine.lock().load_rom_from_file(filepath).unwrap();
    }
}

fn _egui_events_to_machine(i: &InputState, machine: &mut Machine) {
    for event in &i.events {
        // See keymap at https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#keypad
        if let egui::Event::Key {
            key,
            pressed,
            repeat: _,
            modifiers: _,
        } = event
        {
            // TODO: This assume swiss french keyboard
            match key {
                egui::Key::Num1 => machine.key_pressed[1] = *pressed,
                egui::Key::Num2 => machine.key_pressed[2] = *pressed,
                egui::Key::Num3 => machine.key_pressed[3] = *pressed,
                egui::Key::Num4 => machine.key_pressed[0xC] = *pressed,
                egui::Key::Q => machine.key_pressed[4] = *pressed,
                egui::Key::W => machine.key_pressed[5] = *pressed,
                egui::Key::E => machine.key_pressed[6] = *pressed,
                egui::Key::R => machine.key_pressed[0xD] = *pressed,
                egui::Key::A => machine.key_pressed[7] = *pressed,
                egui::Key::S => machine.key_pressed[8] = *pressed,
                egui::Key::D => machine.key_pressed[9] = *pressed,
                egui::Key::F => machine.key_pressed[0xE] = *pressed,
                egui::Key::Y => machine.key_pressed[0xA] = *pressed,
                egui::Key::X => machine.key_pressed[0] = *pressed,
                egui::Key::C => machine.key_pressed[0xB] = *pressed,
                egui::Key::V => machine.key_pressed[0xF] = *pressed,
                _ => {}
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keyboard input
        {
            let mut machine = self.machine.lock();
            ctx.input(|i| _egui_events_to_machine(i, &mut machine));
        }
        // UI drawing
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    self.ui_rom_selection(ui);
                });
                ui.vertical(|ui| {
                    self.ui_display(ui);
                    ui.horizontal(|ui| {
                        self.ui_instruction(ui);
                        self.ui_registers(ui);
                    })
                });
                ui.vertical(|ui| {
                    self.ui_memory(ui);
                    self.ui_keypad(ui);
                })
            })
        });
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        self.machine_thread_tx.send(Message::Exit).unwrap();
        if let Some(handle) = self.machine_thread_handle.take() {
            handle.join().expect("Failed to join");
        }
        if let Some(gl) = gl {
            self.display_renderer.lock().destroy(gl);
        }
    }
}

impl MyApp {
    fn ui_rom_selection(&mut self, ui: &mut egui::Ui) {
        // List all .ch8 files in the roms directory and allow to play them
        if let Ok(paths) = std::fs::read_dir("./roms") {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical(|ui| {
                    for path in paths {
                        if let Ok(path) = path {
                            if path.path().is_dir()
                                || !path.file_name().to_string_lossy().ends_with(".ch8")
                            {
                                continue;
                            }
                            ui.horizontal(|ui| {
                                if ui.button(">").clicked() {
                                    self.play_rom(&format!(
                                        "./roms/{}",
                                        path.file_name().to_string_lossy()
                                    ));
                                }
                                ui.label(path.file_name().to_string_lossy())
                            });
                        } else {
                            ui.label("IO error");
                        }
                    }
                })
            });
        } else {
            ui.label("Couldn't read from ./roms");
        }
    }

    fn ui_display(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                self.custom_painting(ui);
            });
        });
    }

    fn ui_keypad(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label("Keypad");
            egui::Grid::new("keypad").show(ui, |ui| {
                // See keypad layout at https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#keypad
                let keypad_layout = [
                    [1, 2, 3, 0xC],
                    [4, 5, 6, 0xD],
                    [7, 8, 9, 0xE],
                    [0xA, 0, 0xB, 0xF],
                ];
                let machine = self.machine.lock();
                for row in keypad_layout {
                    for key in row {
                        let _ = ui.selectable_label(machine.key_pressed[key], format!("{:X}", key));
                        // TODO: Could handle events through mouse click, but needs to think through the interaction
                        // with keyboard bindings
                    }
                    ui.end_row();
                }
            });
        });
    }

    fn ui_memory(&mut self, ui: &mut egui::Ui) {
        ui.push_id("memory", |ui| {
            // https://github.com/emilk/egui/blob/master/crates/egui_demo_lib/src/demo/table_demo.rs
            ui.vertical(|ui| {
                let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

                let machine = self.machine.lock();
                ui.label(format!("pc={}", machine.program_counter));

                ui.checkbox(&mut self.follow_pc, "Follow PC");

                let mut table = TableBuilder::new(ui)
                    .column(Column::initial(100.0))
                    .column(Column::initial(100.0));

                if self.follow_pc {
                    table = table.scroll_to_row(machine.program_counter, Some(Align::Min));
                }
                table
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("Index");
                        });
                        header.col(|ui| {
                            ui.strong("Content");
                        });
                    })
                    .body(|body| {
                        body.rows(text_height, machine.ram.len(), |index, mut row| {
                            row.col(|ui| {
                                ui.label(index.to_string());
                            });
                            row.col(|ui| {
                                ui.label(format!("{:02x?}", machine.ram[index]));
                            });
                        });
                    });
            });
        });
    }

    fn ui_registers(&mut self, ui: &mut egui::Ui) {
        ui.push_id("registers", |ui| {
            ui.vertical(|ui| {
                let machine = self.machine.lock();
                ui.horizontal(|ui| {
                    ui.label("Flag register");
                    ui.label(format!("{:02x?}", machine.flag_register()));
                });
                ui.label("Registers");
                let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
                let table = TableBuilder::new(ui)
                    .column(Column::initial(100.0))
                    .column(Column::initial(100.0));
                table
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("Register");
                        });
                        header.col(|ui| {
                            ui.strong("Content");
                        });
                    })
                    .body(|mut body| {
                        body.row(text_height, |mut row| {
                            row.col(|ui| {
                                ui.label("index");
                            });
                            row.col(|ui| {
                                ui.label(format!("{:02x?}", machine.index_register));
                            });
                        });
                        body.rows(text_height, machine.registers.len(), |index, mut row| {
                            row.col(|ui| {
                                ui.label(format!("V{:x?}", index));
                            });
                            row.col(|ui| {
                                ui.label(format!("{:02x?}", machine.registers[index]));
                            });
                        });
                    });
            });
        });
    }

    fn ui_instruction(&mut self, ui: &mut egui::Ui) {
        let instruction = self.machine.lock().decode_next_instruction();
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.execution_mode,
                    ExecutionMode::StepByStep,
                    "Step by step",
                );
                ui.selectable_value(
                    &mut self.execution_mode,
                    ExecutionMode::Continuous,
                    "Continuous",
                );
                // TODO: Could use ui.add(egui::SelectableValue).clicked to only change this when clicked
                self.machine_thread_tx
                    .send(Message::ChangeMode(self.execution_mode))
                    .unwrap();
            });

            if self.execution_mode == ExecutionMode::StepByStep
                && ui.button("Execute next").clicked()
            {
                self.machine_thread_tx.send(Message::ExecuteOne).unwrap();
            }
            ui.label(format!("Current instruction: {:?}", instruction));
        });
    }

    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let display_renderer = self.display_renderer.clone();
        let image = self.machine.lock().display.to_image();

        let (rect, _response) = ui.allocate_exact_size(
            egui::Vec2::new(image.width() as f32 * 10.0, image.height() as f32 * 10.0),
            egui::Sense::drag(),
        );

        let callback = egui::PaintCallback {
            rect,
            callback: std::sync::Arc::new(egui_glow::CallbackFn::new(move |_info, painter| {
                display_renderer.lock().paint(painter.gl(), &image);
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
                        vec2(0.0, 1.0),
                        vec2(1.0, 1.0),
                        vec2(0.0, 0.0),
                        vec2(1.0, 0.0)
                    );
                    out vec2 v_uv;
                    void main() {
                        v_uv = uvs[gl_VertexID];
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

    fn paint(&mut self, gl: &glow::Context, image: &RGBAImage) {
        use glow::HasContext as _;
        self.texture.bind(gl, 0);
        self.texture.update(gl, image);
        unsafe {
            gl.use_program(Some(self.program));
            gl.uniform_1_i32(gl.get_uniform_location(self.program, "diffuse").as_ref(), 0);
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        }
    }
}
