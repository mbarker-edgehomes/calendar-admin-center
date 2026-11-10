#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use calendar_admin_center::{app::CalendarAdmin, catalog};
use eframe::egui;

fn main() -> eframe::Result {
    if std::env::args().any(|a| a == "--check-catalog") {
        let commands = catalog::load();
        println!(
            "{} commands; {} available for Exchange Online; {} documented parameters",
            commands.len(),
            commands.iter().filter(|c| c.unavailable.is_none()).count(),
            commands.iter().map(|c| c.parameters.len()).sum::<usize>()
        );
        return Ok(());
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Calendar admin center")
            .with_inner_size([1440.0, 940.0])
            .with_min_inner_size([1000.0, 680.0]),
        ..Default::default()
    };
    let demo = std::env::args().any(|a| a == "--demo");
    eframe::run_native(
        "Calendar admin center",
        options,
        Box::new(move |cc| Ok(Box::new(CalendarAdmin::new(cc, demo)))),
    )
}
