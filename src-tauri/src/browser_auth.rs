//! The system browser is the only user agent used by native sign-in flows.

#[cfg(any(windows, target_os = "linux"))]
mod desktop {
    use std::{
        process::{Command, Stdio},
        thread,
        time::{Duration, Instant},
    };

    const LAUNCH_ERROR: &str =
        "Could not open the system browser. No credentials were transmitted.";

    pub fn open_system_browser(url: &str) -> Result<(), String> {
        let parsed = reqwest::Url::parse(url).map_err(|_| LAUNCH_ERROR)?;
        if parsed.scheme() != "https" || parsed.host_str().is_none() {
            return Err(LAUNCH_ERROR.into());
        }
        let mut child = browser_command(url)?
            .env_remove("OPENAI_API_KEY")
            .env_remove("OPENAI_ACCESS_TOKEN")
            .env_remove("CHATGPT_ACCESS_TOKEN")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| LAUNCH_ERROR)?;
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    return status
                        .success()
                        .then_some(())
                        .ok_or_else(|| LAUNCH_ERROR.into())
                }
                Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(25)),
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(LAUNCH_ERROR.into());
                }
            }
        }
    }

    #[cfg(windows)]
    fn browser_command(url: &str) -> Result<Command, &'static str> {
        use std::{os::windows::process::CommandExt, path::PathBuf};
        use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;

        let powershell = PathBuf::from(std::env::var_os("SystemRoot").ok_or(LAUNCH_ERROR)?)
            .join(r"System32\WindowsPowerShell\v1.0\powershell.exe");
        // URL data never becomes PowerShell source, even when it contains quotes.
        let mut command = Command::new(powershell);
        command
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "$url = $env:OVERLAY_FORGE_OAUTH_URL; Remove-Item Env:OVERLAY_FORGE_OAUTH_URL; Start-Process -FilePath $url -ErrorAction Stop",
            ])
            .env("OVERLAY_FORGE_OAUTH_URL", url)
            .creation_flags(CREATE_NO_WINDOW);
        Ok(command)
    }

    #[cfg(target_os = "linux")]
    fn browser_command(url: &str) -> Result<Command, &'static str> {
        let mut command = Command::new("/usr/bin/gio");
        command.args(["open", url]);
        Ok(command)
    }

    #[cfg(all(test, target_os = "linux"))]
    mod tests {
        use super::*;
        use std::ffi::OsStr;

        #[test]
        fn passes_the_entire_oauth_url_to_gio_as_one_argument() {
            let url =
                "https://accounts.google.com/o/oauth2/v2/auth?state=test&prompt=select_account";
            let command = browser_command(url).unwrap();
            assert_eq!(command.get_program(), "/usr/bin/gio");
            assert_eq!(
                command.get_args().collect::<Vec<_>>(),
                [OsStr::new("open"), OsStr::new(url)]
            );
        }

        #[test]
        fn rejects_urls_without_https_before_launching() {
            for url in ["http://example.com", "file:///tmp/example", "invalid"] {
                assert_eq!(open_system_browser(url).unwrap_err(), LAUNCH_ERROR);
            }
        }
    }
}

#[cfg(any(windows, target_os = "linux"))]
pub use desktop::open_system_browser;

#[cfg(not(any(windows, target_os = "linux")))]
pub fn open_system_browser(_url: &str) -> Result<(), String> {
    Err("Browser sign-in is currently available on Windows and Linux.".into())
}
