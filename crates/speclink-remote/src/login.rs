//! Driving a device authorization from initiation to a verdict.
//!
//! The loop lives here rather than in a front-end so the CLI and the desktop
//! app reach the same outcome and write the same keyring entries — a login on
//! either one is a login for both (cli-desktop-credential-sharing).
//!
//! Terminal, browser and clock are injected through [`DeviceLoginIo`]: tests
//! need a device flow that resolves instantly, and a CLI needs one that prints
//! and waits. The device flow's HTTP primitives stay in [`crate::device`].

use crate::credentials::{CredentialKind, CredentialStore};
use crate::device::{self, InitiateOutcome};
use speclink_protocol::device::DeviceTokenStatus;
use std::path::Path;

/// How much to widen the polling interval when the server asks for room.
const SLOW_DOWN_STEP_SECS: u64 = 5;

/// The terminal, browser and clock a device login needs.
pub trait DeviceLoginIo {
    /// Show the verification URL and user code. Always called, even when a
    /// browser opens: approving from a phone or a second machine is a
    /// first-class path, not a fallback.
    fn announce(&self, verification_uri: &str, user_code: &str);
    /// Best-effort browser launch; `false` simply means the user approves
    /// from what was announced.
    fn open_browser(&self, url: &str) -> bool;
    fn sleep_secs(&self, secs: u64);
}

/// How a device login ended. Denial, expiry and an absent device flow are
/// verdicts, not errors — each needs its own message, and only a genuinely
/// broken server should read as a failure.
#[derive(Debug)]
pub enum DeviceLoginOutcome {
    Approved { display: String },
    Denied,
    Expired,
    /// The server offers no device flow — the PAT fallback signal.
    Unsupported,
}

/// Run a device authorization to its verdict, storing the credentials on
/// approval.
///
/// Both credentials are written before the identity lookup: the refresh
/// credential is what survives this process, and the access token beside it is
/// what keeps the next command from spending a rotation.
pub fn device_login(
    origin: &str,
    credentials: &dyn CredentialStore,
    lock_dir: &Path,
    io: &dyn DeviceLoginIo,
) -> Result<DeviceLoginOutcome, String> {
    let auth = match device::initiate(origin).map_err(|e| e.to_string())? {
        InitiateOutcome::Supported(auth) => auth,
        InitiateOutcome::Unsupported => return Ok(DeviceLoginOutcome::Unsupported),
    };

    io.announce(&auth.verification_uri, &auth.user_code);
    let sep = if auth.verification_uri.contains('?') { '&' } else { '?' };
    io.open_browser(&format!(
        "{}{sep}user_code={}",
        auth.verification_uri, auth.user_code
    ));

    let mut interval = auth.interval.max(1);
    let mut waited = 0u64;
    loop {
        let resp = device::poll(origin, &auth.device_code).map_err(|e| e.to_string())?;
        match resp.status {
            DeviceTokenStatus::Approved => {
                let access = resp
                    .access_token
                    .ok_or_else(|| "server 回報已核准卻未附 access token".to_string())?;
                let refresh = resp
                    .refresh_token
                    .ok_or_else(|| "server 回報已核准卻未附 refresh credential".to_string())?;
                // Under the rotation lock: another process mid-rotation would
                // otherwise write back a credential from the family this login
                // just replaced.
                crate::refresh::with_rotation_lock(lock_dir, || {
                    credentials.set(origin, CredentialKind::Refresh, &refresh)?;
                    crate::refresh::store_cached_bearer(
                        origin,
                        credentials,
                        &access,
                        resp.expires_in.unwrap_or(0),
                    )
                })?;
                let who = device::whoami(origin, &access).map_err(|e| e.to_string())?;
                return Ok(DeviceLoginOutcome::Approved { display: who.user.name });
            }
            DeviceTokenStatus::Denied => return Ok(DeviceLoginOutcome::Denied),
            DeviceTokenStatus::Expired => return Ok(DeviceLoginOutcome::Expired),
            DeviceTokenStatus::SlowDown => interval += SLOW_DOWN_STEP_SECS,
            DeviceTokenStatus::Pending => {}
        }

        // The server's own expiry is the ceiling; without it a lost approval
        // page would poll until the user gave up.
        waited += interval;
        if waited > auth.expires_in {
            return Ok(DeviceLoginOutcome::Expired);
        }
        io.sleep_secs(interval);
    }
}

/// Drop every credential this origin holds locally: the device login and the
/// keyring PAT alike. Used by logout, where the point is that nothing usable
/// is left behind.
pub fn clear_all_local_credentials(origin: &str, credentials: &dyn CredentialStore) {
    let _ = credentials.delete(origin, CredentialKind::Refresh);
    let _ = credentials.delete(origin, CredentialKind::Bearer);
    let _ = credentials.delete(origin, CredentialKind::Pat);
}
