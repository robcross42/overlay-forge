//! The system browser is the only user agent used by native sign-in flows.

#[cfg(windows)]
pub fn open_system_browser(url: &str) -> Result<(), String> {
    use std::{
        os::windows::process::CommandExt,
        path::PathBuf,
        process::{Command, Stdio},
        thread,
        time::{Duration, Instant},
    };
    use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;

    let error = "Could not open the system browser. No credentials were transmitted.";
    let parsed = reqwest::Url::parse(url).map_err(|_| error)?;
    if parsed.scheme() != "https" || parsed.host_str().is_none() {
        return Err(error.into());
    }
    let powershell = PathBuf::from(std::env::var_os("SystemRoot").ok_or(error)?)
        .join(r"System32\WindowsPowerShell\v1.0\powershell.exe");
    // URL data never becomes PowerShell source, even when it contains quotes.
    let mut child = Command::new(powershell)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "$url = $env:OVERLAY_FORGE_OAUTH_URL; Remove-Item Env:OVERLAY_FORGE_OAUTH_URL; Start-Process -FilePath $url -ErrorAction Stop",
        ])
        .env("OVERLAY_FORGE_OAUTH_URL", url)
        .env_remove("OPENAI_API_KEY")
        .env_remove("OPENAI_ACCESS_TOKEN")
        .env_remove("CHATGPT_ACCESS_TOKEN")
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| error)?;
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.success().then_some(()).ok_or_else(|| error.into()),
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(25)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error.into());
            }
        }
    }
}

#[cfg(not(windows))]
pub fn open_system_browser(_url: &str) -> Result<(), String> {
    Err("Browser sign-in is currently available on Windows.".into())
}
