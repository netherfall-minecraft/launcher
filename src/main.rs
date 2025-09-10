slint::include_modules!();

mod app;
mod ui;
mod core;
mod launcher;


fn main() -> Result<(), slint::PlatformError> {
    core::logging::init();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    let ui = AppWindow::new()?;
    let weak = ui.as_weak();

    rt.spawn(async move {
        if let Err(e) = core::http::warmup().await {
            tracing::warn!("warmup failed: {e:#}");
        }
        // вызов в UI
        if let Some(app) = weak.upgrade() {
            slint::invoke_from_event_loop(move || {
            }).ok();
        }
    });

    ui.run()
}
