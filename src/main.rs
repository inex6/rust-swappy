use clap::Parser;
use eframe::egui;
use image::{DynamicImage, ImageReader};
use std::io::{self, Read};
use std::path::PathBuf;

// --- Main Entry Point --- //

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[LOG] Starting rust-swappy...");

    // 1. Parse CLI args and load image data
    let args = Args::parse();
    println!("[LOG] Parsed arguments: {:?}", args);

    let image_data = match args.file {
        // A real file path was provided that is not "-".
        Some(path) if path.to_str() != Some("-") => {
            println!("[LOG] Reading from file: {:?}", path);
            std::fs::read(path)?
        }
        // This handles both `None` (no -f arg) and `Some("-")`
        _ => {
            println!("[LOG] Reading from stdin...");
            let mut buffer = Vec::new();
            io::stdin().read_to_end(&mut buffer)?;
            buffer
        }
    };
    println!("[LOG] Read {} bytes of image data.", image_data.len());

    if image_data.is_empty() {
        println!("[ERROR] No image data received. Did you pipe anything to stdin?");
        return Ok(()); // Exit gracefully
    }

 let image = ImageReader::new(std::io::Cursor::
     new(image_data)).with_guessed_format()?.decode()?;
    println!("[LOG] Decoded image with dimensions: {}x{}", image.width(), image.height());

    // 2. Set up eframe options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([image.width() as f32, image.height() as f32])
            .with_resizable(true),
        ..Default::default()
    };

    // 3. Run the eframe application
    println!("[LOG] Starting eframe GUI...");
    eframe::run_native(
        "Rust Swappy (egui)",
        options,
        Box::new(|_cc| Box::new(Editor::new(image))),
    )?;
    println!("[LOG] eframe GUI finished.");

    Ok(())
}

// --- CLI Arg Struct --- //

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    file: Option<PathBuf>,
}

// --- eframe Application State --- //

struct Editor {
    /// The image texture handle that egui uses to display the image
    texture: Option<egui::TextureHandle>,
    /// The original image data, kept for potential future use (like saving)
    image: DynamicImage,
}

impl Editor {
    /// Constructor for our editor app
    fn new(image: DynamicImage) -> Self {
        println!("[LOG] Creating new Editor app state.");
        Self {
            texture: None,
            image,
        }
    }
}

// --- eframe Application Trait Implementation --- //

impl eframe::App for Editor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let is_first_frame = self.texture.is_none();
        if is_first_frame {
            println!("[LOG] Editor::update running for the first time.");
        }

        // On the first frame, load the image data into a texture
        let texture = self.texture.get_or_insert_with(|| {
            println!("[LOG] Loading image into egui texture...");
            let size = [self.image.width() as usize, self.image.height() as usize];
            let image_buffer = self.image.to_rgba8();
            let pixels = image_buffer.as_flat_samples();
            let egui_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

            ctx.load_texture("swappy-image", egui_image, Default::default())
        });

        if is_first_frame {
            println!("[LOG] Texture created successfully.");
        }

        // Show the image in the central panel
        egui::CentralPanel::default().show(ctx, |ui| {
            // Add the image to the UI, centered
            ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                ui.image(&*texture);
            });
        });

        // Keep repainting to see if the update loop is running
        ctx.request_repaint();
    }
}
