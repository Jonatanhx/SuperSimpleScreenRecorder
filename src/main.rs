use std::io::{self, Write};
use std::time::Instant;

use eframe::egui;

use windows_capture::capture::{CaptureControl, Context, GraphicsCaptureApiHandler};
use windows_capture::encoder::{
    AudioSettingsBuilder, ContainerSettingsBuilder, VideoEncoder, VideoSettingsBuilder,
};
use windows_capture::frame::Frame;
use windows_capture::graphics_capture_api::{InternalCaptureControl};
use windows_capture::monitor::Monitor;
use windows_capture::settings::{
    ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
    MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
};

struct Capture {
    encoder: Option<VideoEncoder>,
    start: Instant,
}

impl GraphicsCaptureApiHandler for Capture {
    type Flags = String;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        println!("Created with flags: {}", ctx.flags);

        let encoder = VideoEncoder::new(
            VideoSettingsBuilder::new(1920, 1080),
            AudioSettingsBuilder::default().disabled(true),
            ContainerSettingsBuilder::default(),
            "video.mp4",
        )?;

        Ok(Self { encoder: Some(encoder), start: Instant::now()})
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        _capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        print!("\rRecording for: {} seconds", self.start.elapsed().as_secs());
        io::stdout().flush()?;

        self.encoder.as_mut().unwrap().send_frame(frame)?;

        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        println!("Capture session ended");

        Ok(())
    }
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_transparent(true),
        ..Default::default()
    };
    eframe::run_native("Super Simple Screen Recorder", options, Box::new(|cc| Box::new(MyEguiApp::new(cc)))).expect("Failed to run app");
}

fn record() -> CaptureControl<Capture, Box<dyn std::error::Error + Send + Sync>>
 {
    let primary_monitor = Monitor::primary().expect("There is no primary monitor");
    let settings = Settings::new(
        primary_monitor,
        CursorCaptureSettings::Default,
        DrawBorderSettings::Default,
        SecondaryWindowSettings::Default,
        MinimumUpdateIntervalSettings::Default,
        DirtyRegionSettings::Default,
        ColorFormat::Rgba8,
        "Yea this works".to_string(),
    );
    
    Capture::start_free_threaded(settings).expect("Screen capture failed")
}

#[derive(Default)]
struct MyEguiApp {
    capture_control: Option<CaptureControl<Capture, Box<dyn std::error::Error + Send + Sync>>>,
}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
}

impl eframe::App for MyEguiApp {
   fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
       egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
        ui.heading("Hello World!");
        
        let response = ui.add(egui::Slider::new(&mut 5, 0..=100));
        response.on_hover_text("Im a slider!");

        let btn1 = ui.button("start");
        let btn2 = ui.button("stop");

        if btn1.clicked() {
            self.capture_control = Some(record());
        };

        if btn2.clicked() {
            if let Some(control) = self.capture_control.take() {
                control.stop().expect("Failed to stop recording");
            }
        }
       });
   }
}