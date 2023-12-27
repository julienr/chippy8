use eframe::glow;
use eframe::glow::HasContext;

pub struct RGBAImage {
    _rgba: Vec<u8>,
    _width: usize,
    _height: usize,
}

impl RGBAImage {
    pub fn new(data: Vec<u8>, width: usize, height: usize) -> RGBAImage {
        RGBAImage {
            _rgba: data,
            _width: width,
            _height: height,
        }
    }

    pub fn data(&self) -> &[u8] {
        &self._rgba
    }
    pub fn width(&self) -> usize {
        self._width
    }
    pub fn height(&self) -> usize {
        self._height
    }
}

pub struct Texture {
    texture: glow::Texture,
    width: usize,
    height: usize,
}

impl Texture {
    fn new(gl: &glow::Context, width: usize, height: usize, pixels: &[u8]) -> Self {
        let texture: glow::Texture;
        unsafe {
            texture = gl.create_texture().expect("Failed to create texture");
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
        }
        unsafe {
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA8 as i32,
                width as i32,
                height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(pixels),
            );
        }

        Self {
            texture,
            width,
            height,
        }
    }

    pub fn checkerboard(gl: &glow::Context, width: usize, height: usize) -> Self {
        let square_size = 4;
        let mut arr: Vec<u8> = vec![0; width * height * 4];
        for i in 0..height {
            for j in 0..width {
                let color = if (i / square_size) % 2 != (j / square_size) % 2 {
                    (0, 0, 0, 0)
                } else {
                    (255, 255, 255, 255)
                };
                arr[(i * height + j) * 4] = color.0;
                arr[(i * height + j) * 4 + 1] = color.1;
                arr[(i * height + j) * 4 + 2] = color.2;
                arr[(i * height + j) * 4 + 3] = color.3;
            }
        }
        Texture::new(gl, width, height, arr.as_slice())
    }

    pub fn bind(&self, gl: &glow::Context, texture_unit: u32) {
        unsafe {
            gl.active_texture(texture_unit);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
        }
    }

    /// This MUST only be called after bind()
    /// => TODO: Enforce this
    pub fn update(&mut self, gl: &glow::Context, img: &RGBAImage) {
        if img.width() != self.width || img.height() != self.height {
            panic!(
                "Incompatible width/height: {}x{} vs {}x{}",
                img.width(),
                img.height(),
                self.width,
                self.height
            );
        }
        unsafe {
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA8 as i32,
                self.width as i32,
                self.height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(img.data()),
            );
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn destroy(&self, gl: &glow::Context) {
        use glow::HasContext as _;
        unsafe {
            gl.delete_texture(self.texture);
        }
    }
}
