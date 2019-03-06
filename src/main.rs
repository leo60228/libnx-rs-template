use cfg_if::cfg_if;

pub mod util;

cfg_if! {
    if #[cfg(target_os = "horizon")] {
        use ::input as einp;
        use std::ptr;
        use libnx_rs::libnx::consoleInit;
        use libnx_rs::libnx::gfxInitDefault;
        use std::fs::File;
        use std::fs::OpenOptions;
        use std::fs::read;
        use std::os::unix::io::AsRawFd;
        use std::io;
        use std::io::Write;
/*        use conrod::text::FontCollection;
        use conrod::event;
        use conrod::input;
        use libnx_rs_window::NxGlWindow;
        use piston_window::texture::UpdateTexture;
        use piston_window::OpenGL;
        use piston_window::{G2d, G2dTexture, TextureSettings};
        use piston_window::{PistonWindow, Window, WindowSettings};*/
        use util::*;
        use std::path::PathBuf;
        use libc;

        // mod support;

        pub fn main() {
            if let Err(_) = redirect_stderr("conrod-stderr.txt") {
                return;
            }

            eprintln!("redirected stderr");

            if let Err(err) = redirect_stdout("conrod-stdout.txt") {
                eprintln!("Redirection error: {:?}", err);
                return;
            }

            eprintln!("redirected stdout");

            unsafe {
                gfxInitDefault();
                eprintln!("initialized gfx");
                consoleInit(ptr::null_mut());
                eprintln!("initialized console");
            }

            let nca = get_tid_nca(TitleId(0x01002B30028F6000)).unwrap();
            println!("sdk path: {:?}", nca.0);
            let path = nca.mount();
            println!("libnx path: {:?})", path);
            let outp = PathBuf::from("celeste_dump");

            extract_nca(&path, &outp).unwrap();
/*
            const WIDTH: u32 = support::WIN_W;
            const HEIGHT: u32 = support::WIN_H;

            // Construct the window.
            let mut window: PistonWindow<NxGlWindow> = match WindowSettings::new("", [WIDTH, HEIGHT])
                .opengl(OpenGL::V3_2) // If not working, try `OpenGL::V2_1`.
                .samples(4)
                .exit_on_esc(true)
                .vsync(true)
                .build()
            {
                Ok(w) => w,
                Err(_) => {
                    return;
                }
            };

            // construct our `Ui`.
            let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64])
                .theme(support::theme())
                .build();

            // Add a `Font` to the `Ui`'s `font::Map` from file.
            use std::path::Path;
            let font_data = match read(Path::new("assets/NotoSans-Regular.ttf")) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading font data file: {}", e);
                    return;
                }
            };
            let collection = match FontCollection::from_bytes(font_data) {
                Ok(fc) => fc,
                Err(e) => {
                    eprintln!("error constructing a FontCollection from bytes: {}", e);
                    return;
                }
            };

            let font = match collection.into_font() {
                // only succeeds if collection consists of one font
                Ok(f) => f,
                Err(e) => {
                    eprintln!("error turning FontCollection into a Font: {}", e);
                    return;
                }
            };

            ui.fonts.insert(font);

            // Create a texture to use for efficiently caching text on the GPU.
            let mut text_vertex_data = Vec::new();
            let (mut glyph_cache, mut text_texture_cache) = {
                const SCALE_TOLERANCE: f32 = 0.1;
                const POSITION_TOLERANCE: f32 = 0.1;
                let cache =
                    conrod::text::GlyphCache::new(WIDTH, HEIGHT, SCALE_TOLERANCE, POSITION_TOLERANCE);
                let buffer_len = WIDTH as usize * HEIGHT as usize;
                let init = vec![128; buffer_len];
                let settings = TextureSettings::new();
                let factory = &mut window.factory;
                let texture =
                    G2dTexture::from_memory_alpha(factory, &init, WIDTH, HEIGHT, &settings).unwrap();
                (cache, texture)
            };

            // Instantiate the generated list of widget identifiers.
            let ids = support::Ids::new(ui.widget_id_generator());

            // Empty image map
            // TODO: check if this is necessary
            let image_map = conrod::image::Map::new();

            //
            // A demonstration of some state that we'd like to control with the App.
            let mut app = support::DemoApp::new();

            // Poll events from the window.
            while let Some(event) = window.next() {
                // Convert the piston event to a conrod event.
                let size = window.size();
                let (win_w, win_h) = (size.width as conrod::Scalar, size.height as conrod::Scalar);
                if let Some(e) = conrod::backend::piston::event::convert(event.clone(), win_w, win_h) {
                    match e {
                        event::Input::Press(input::Button::Hat(ht)) => {
                            let rootid = ids.canvas;
                            if ht.state == einp::HatState::Up {
                                let scrl = [-80.0, -80.0];
                                ui.scroll_widget(rootid, scrl);
                            } else if ht.state == einp::HatState::Down {
                                let scrl = [80.0, 80.0];
                                ui.scroll_widget(rootid, scrl);
                            }
                            ui.handle_event(e);
                        }
                        e => {
                            ui.handle_event(e);
                        }
                    }
                }

                {
                    let mut ui = ui.set_widgets();
                    support::gui(&mut ui, &ids, &mut app);
                };

                window.draw_2d(&event, |context, graphics| {
                    if let Some(primitives) = ui.draw_if_changed() {
                        // A function used for caching glyphs to the texture cache.
                        let cache_queued_glyphs = |graphics: &mut G2d,
                                                   cache: &mut G2dTexture,
                                                   rect: conrod::text::rt::Rect<u32>,
                                                   data: &[u8]| {
                            let offset = [rect.min.x, rect.min.y];
                            let size = [rect.width(), rect.height()];
                            let format = piston_window::texture::Format::Rgba8;
                            let encoder = &mut graphics.encoder;
                            text_vertex_data.clear();
                            text_vertex_data.extend(data.iter().flat_map(|&b| vec![255, 255, 255, b]));
                            let res = UpdateTexture::update(
                                cache,
                                encoder,
                                format,
                                &text_vertex_data[..],
                                offset,
                                size,
                            )
                            .expect("failed to update texture");
                            res
                        };

                        // Specify how to get the drawable texture from the image. In this case, the image
                        // *is* the texture.
                        fn texture_from_image<T>(img: &T) -> &T {
                            img
                        }

                        // Draw the conrod `render::Primitives`.
                        conrod::backend::piston::draw::primitives(
                            primitives,
                            context,
                            graphics,
                            &mut text_texture_cache,
                            &mut glyph_cache,
                            &image_map,
                            cache_queued_glyphs,
                            texture_from_image,
                        );
                    }
                });
            }*/
        }

        pub fn redirect_stdout (filename : &str) -> Result<File, io::Error> {
            let mut outfile = OpenOptions::new()
                .write(true)
                .create(true)
                .open(filename)?;
            outfile.write_fmt(format_args!("Redirecting standard output to {}.", filename))?;
            let raw_fd = outfile.as_raw_fd();
            let new_fd = unsafe {
                libc::fflush(0 as *mut libc::FILE);
                libc::dup2(raw_fd, libc::STDOUT_FILENO)
            };
            if new_fd != libc::STDOUT_FILENO {
                Err(io::Error::new(io::ErrorKind::Other, format!("Could not call dup2. Ended up redirecting fd {} to {} instead of {}.", raw_fd, new_fd, libc::STDOUT_FILENO)))
            }
            else {
                Ok(outfile)
            }
        }

        pub fn redirect_stderr (filename : &str) -> Result<File, io::Error> {
            let mut outfile = OpenOptions::new()
                .write(true)
                .create(true)
                .open(filename)?;
            outfile.write_fmt(format_args!("Redirecting standard error to {}.\n", filename))?;
            let raw_fd = outfile.as_raw_fd();
            let new_fd = unsafe {
                libc::fflush(0 as *mut libc::FILE);
                libc::dup2(raw_fd, libc::STDERR_FILENO)
            };
            if new_fd != libc::STDERR_FILENO {
                Err(io::Error::new(io::ErrorKind::Other, format!("Could not call dup2. Ended up redirecting fd {} to {} instead of {}.", raw_fd, new_fd, libc::STDERR_FILENO)))
            }
            else {
                Ok(outfile)
            }
        }
    }
}
