//! Cancellable Host setup. Keep this guard alongside the host session for the
//! lifetime of its invite; dropping it stops only its own internet helper.
use super::{
    invite::Invite,
    transport::{self, Conditions, Peer},
    tunnel::{self, Tunnel},
};
use anyhow::{ensure, Result};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, TryRecvError},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    PreparingHelper,
    OpeningTunnel,
    Ready,
    Failed(String),
    Stopped,
}
struct Shared {
    cancel: AtomicBool,
    status: Mutex<Status>,
}
impl Shared {
    fn set(&self, status: Status) {
        *self.status.lock().unwrap() = status;
    }
    fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Acquire)
    }
}

pub struct Hosting {
    shared: Arc<Shared>,
    result: Receiver<(Peer, Invite)>,
}
impl Hosting {
    pub fn start(
        profile: PathBuf,
        helper_cache: PathBuf,
        internet: bool,
        conditions: Conditions,
    ) -> Result<Self> {
        let shared = Arc::new(Shared {
            cancel: AtomicBool::new(false),
            status: Mutex::new(Status::OpeningTunnel),
        });
        let (sender, result) = mpsc::sync_channel(1);
        let worker = shared.clone();
        thread::Builder::new()
            .name("coop-invite".into())
            .spawn(move || {
                let result = (|| -> Result<()> {
                    ensure!(!worker.cancelled(), "Hosting cancelled.");
                    let (peer, local) = transport::host(conditions)?;
                    let mut tunnel = None;
                    let invite = if internet {
                        worker.set(Status::PreparingHelper);
                        let helper = tunnel::ensure_helper(&helper_cache, &worker.cancel)?;
                        ensure!(!worker.cancelled(), "Hosting cancelled.");
                        worker.set(Status::OpeningTunnel);
                        let mut opening =
                            Tunnel::start(&helper, local.origin(), &profile.join("tunnel"))?;
                        let origin = loop {
                            ensure!(!worker.cancelled(), "Hosting cancelled.");
                            if let Some(origin) = opening.poll()? {
                                break origin.to_owned();
                            }
                            thread::sleep(Duration::from_millis(50));
                        };
                        tunnel = Some(opening);
                        local.with_origin(&origin)?
                    } else {
                        local
                    };
                    ensure!(!worker.cancelled(), "Hosting cancelled.");
                    sender
                        .send((peer, invite))
                        .map_err(|_| anyhow::anyhow!("Hosting cancelled."))?;
                    worker.set(Status::Ready);
                    while !worker.cancelled() {
                        if let Some(tunnel) = &mut tunnel {
                            tunnel.poll()?;
                        }
                        thread::sleep(Duration::from_millis(50));
                    }
                    Ok(()) // owned Tunnel drops here, including all error exits
                })();
                match result {
                    Err(error) if !worker.cancelled() => {
                        worker.set(Status::Failed(error.to_string()))
                    }
                    _ => worker.set(Status::Stopped),
                }
            })?;
        Ok(Self { shared, result })
    }
    pub fn status(&self) -> Status {
        self.shared.status.lock().unwrap().clone()
    }
    /// The caller moves the ready listener into Host and retains this guard.
    pub fn take_ready(&self) -> Result<Option<(Peer, Invite)>> {
        ensure!(!self.shared.cancelled(), "Hosting cancelled.");
        match self.result.try_recv() {
            Ok(value) => Ok(Some(value)),
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => match self.status() {
                Status::Failed(reason) => anyhow::bail!("{reason}"),
                Status::Stopped => anyhow::bail!("Hosting stopped."),
                _ => Ok(None),
            },
        }
    }
    pub fn cancel(&self) {
        self.shared.cancel.store(true, Ordering::Release);
    }
}
impl Drop for Hosting {
    fn drop(&mut self) {
        self.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    #[test]
    fn dropping_or_cancelling_setup_releases_its_listener() {
        let hosting =
            Hosting::start(PathBuf::new(), PathBuf::new(), false, Conditions::default()).unwrap();
        let started = Instant::now();
        let (peer, invite) = loop {
            if let Some(ready) = hosting.take_ready().unwrap() {
                break ready;
            }
            assert!(started.elapsed() < Duration::from_secs(3));
            thread::sleep(Duration::from_millis(5));
        };
        let address = invite.origin().strip_prefix("http://").unwrap().to_owned();
        drop(peer);
        hosting.cancel();
        assert!(hosting.take_ready().is_err());
        while hosting.status() != Status::Stopped {
            assert!(started.elapsed() < Duration::from_secs(3));
            thread::sleep(Duration::from_millis(5));
        }
        // Listener worker may take a scheduling quantum to notice Peer::drop.
        loop {
            if let Ok(listener) = std::net::TcpListener::bind(&address) {
                drop(listener);
                break;
            }
            assert!(started.elapsed() < Duration::from_secs(3));
            thread::sleep(Duration::from_millis(5));
        }
    }
}
