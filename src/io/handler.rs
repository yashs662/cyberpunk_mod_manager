use std::sync::Arc;
use crate::app::App;
use eyre::Result;
use log::{
    error,
    info,
};

use super::IoEvent;

/// In the IO thread, we handle IO event without blocking the UI thread
pub struct IoAsyncHandler {
    app: Arc<tokio::sync::Mutex<App>>,
}

impl IoAsyncHandler {
    pub fn new(app: Arc<tokio::sync::Mutex<App>>) -> Self {
        Self { app }
    }

    /// We could be async here
    pub async fn handle_io_event(&mut self, io_event: IoEvent) {
        let result = match io_event {
            IoEvent::Initialize => self.do_initialize().await,
            IoEvent::GoRight => self.go_right().await,
            IoEvent::GoLeft => self.go_left().await,
            IoEvent::GoUp => self.go_up().await,
            IoEvent::GoDown => self.go_down().await,
        };

        if let Err(err) = result {
            error!("Oops, something wrong happen: {:?}", err);
        }

        let mut app = self.app.lock().await;
        app.loaded();
    }

    /// We use dummy implementation here, just wait 1s
    async fn do_initialize(&mut self) -> Result<()> {
        info!("🚀 Initialize the application");
        let mut app = self.app.lock().await;
        app.initialized(); // we could update the app state
        info!("👍 Application initialized");
        Ok(())
    }

    async fn go_right(&mut self) -> Result<()> {
        Ok(())
    }

    async fn go_left(&mut self) -> Result<()> {
        Ok(())
    }

    async fn go_up(&mut self) -> Result<()> {
        Ok(())
    }

    async fn go_down(&mut self) -> Result<()> {
        Ok(())
    }
}