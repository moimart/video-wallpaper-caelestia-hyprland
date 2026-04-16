mod app;
mod config;
mod gallery;
mod services;
mod settings;
mod theming;
mod util;
mod window;

fn main() -> glib::ExitCode {
    // Use NGL renderer to avoid Vulkan memory exhaustion when playing video previews
    if std::env::var("GSK_RENDERER").is_err() {
        // Safety: called before any threads are spawned (single-threaded at this point)
        unsafe { std::env::set_var("GSK_RENDERER", "gl") };
    }
    env_logger::init();
    let app = app::VideoWallpaperApp::new();
    app.run()
}
