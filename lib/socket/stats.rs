use {
    core::fmt,
    std::{
        fmt::{Debug, Formatter},
        sync::{atomic::AtomicU64, Arc},
    },
};

/// `ServerStats` is a struct that holds the server statistics such as total clients and current
/// clients; only used for debugging purposes.
#[derive(Default)]
pub struct ServerStats {
    total_clients: Arc<AtomicU64>,
    current_clients: Arc<AtomicU64>,
}

impl ServerStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// increments the total clients and current clients count.
    pub fn inc_current_clients(&self) {
        self.current_clients.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.total_clients.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// decrements the current clients count.
    pub fn dec_current_clients(&self) {
        self.current_clients.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Debug for ServerStats {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServerStats")
            .field("total_clients", &self.total_clients)
            .field("current_clients", &self.current_clients)
            .finish()
    }
}
