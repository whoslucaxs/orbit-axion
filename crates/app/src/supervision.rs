use std::{process::Command, time::Duration};
/// Separate supervisor process; the resident process remains unelevated.
pub fn run(args: &[String]) -> Result<Option<i32>, String> {
    if !args.iter().any(|a| a == "--watchdog") {
        return Ok(None);
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let forwarded: Vec<_> = args
        .iter()
        .skip(1)
        .filter(|a| a.as_str() != "--watchdog")
        .collect();
    for attempt in 0..=3 {
        let mut child = Command::new(&exe);
        child.args(&forwarded);
        if attempt == 3 {
            child.arg("--safe-mode");
        }
        let status = child.status().map_err(|e| e.to_string())?;
        if status.success() {
            return Ok(Some(0));
        }
        if attempt < 3 {
            std::thread::sleep(Duration::from_secs(1 << attempt));
        }
    }
    Err("Orbit falhou repetidamente; reinicialização automática encerrada. Use --safe-mode e consulte os logs locais.".into())
}
