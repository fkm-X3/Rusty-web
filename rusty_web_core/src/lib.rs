use serde::Deserialize;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize, Position, Size};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};
use wry::{Rect, WebView, WebViewBuilder};

const TOOLBAR_HEIGHT: f64 = 90.0;

#[derive(Debug)]
enum BrowserEvent {
    Navigate(String),
    Settings,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
enum IpcMessage {
    Navigate { url: String },
    Settings,
}

struct App {
    window: Option<Window>,
    toolbar_webview: Option<WebView>,
    content_webview: Option<WebView>,
    proxy: EventLoopProxy<BrowserEvent>,
    current_url: String,
}

impl App {
    fn new(proxy: EventLoopProxy<BrowserEvent>) -> Self {
        Self {
            window: None,
            toolbar_webview: None,
            content_webview: None,
            proxy,
            current_url: "https://www.google.com".to_string(),
        }
    }

    fn resize_webviews(&mut self, width: f64, height: f64) {
        if let Some(webview) = &self.toolbar_webview {
            let _ = webview.set_bounds(Rect {
                position: Position::Logical(LogicalPosition::new(0.0, 0.0)),
                size: Size::Logical(LogicalSize::new(width, TOOLBAR_HEIGHT)),
            });
        }
        if let Some(webview) = &self.content_webview {
            let _ = webview.set_bounds(Rect {
                position: Position::Logical(LogicalPosition::new(0.0, TOOLBAR_HEIGHT)),
                size: Size::Logical(LogicalSize::new(width, (height - TOOLBAR_HEIGHT).max(1.0))),
            });
        }
    }

    fn update_toolbar_url(&self, url: &str) {
        if let Some(toolbar) = &self.toolbar_webview {
            let script = format!(
                "if (typeof updateUrlFromBrowser === 'function') {{ updateUrlFromBrowser('{}'); }}",
                url.replace('\'', "\\'").replace('\\', "\\\\")
            );
            let _ = toolbar.evaluate_script(&script);
        }
    }
}

impl ApplicationHandler<BrowserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("Rusty Web Browser")
                    .with_inner_size(LogicalSize::new(1024.0, 768.0)),
            )
            .expect("Failed to create window");

        let size = window.inner_size().to_logical::<f64>(window.scale_factor());
        let width = size.width;
        let height = size.height;
        
        // --- Toolbar WebView ---
        let proxy = self.proxy.clone();
        let toolbar_html = include_str!("toolbar.html");
        
        let toolbar_webview = WebViewBuilder::new()
            .with_html(toolbar_html)
            .with_bounds(Rect {
                position: Position::Logical(LogicalPosition::new(0.0, 0.0)),
                size: Size::Logical(LogicalSize::new(width, TOOLBAR_HEIGHT)),
            })
            .with_ipc_handler(move |msg| {
                if let Ok(message) = serde_json::from_str::<IpcMessage>(&msg.body()) {
                     match message {
                        IpcMessage::Navigate { url } => {
                             let _ = proxy.send_event(BrowserEvent::Navigate(url));
                        }
                        IpcMessage::Settings => {
                             let _ = proxy.send_event(BrowserEvent::Settings);
                        }
                    }
                }
            })
            .build(&window)
            .expect("Failed to create toolbar webview");

        // --- Content WebView ---
        let content_webview = WebViewBuilder::new()
            .with_url("https://www.google.com")
            .with_bounds(Rect {
                position: Position::Logical(LogicalPosition::new(0.0, TOOLBAR_HEIGHT)),
                size: Size::Logical(LogicalSize::new(width, (height - TOOLBAR_HEIGHT).max(1.0))),
            })
            .with_navigation_handler(|url| {
                println!("Navigation to: {}", url);
                true
            })
            .build(&window)
            .expect("Failed to create content webview");

        self.window = Some(window);
        self.toolbar_webview = Some(toolbar_webview);
        self.content_webview = Some(content_webview);
        
        // Initialize toolbar with current URL
        self.update_toolbar_url(&self.current_url);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(window) = &self.window {
                    let scale_factor = window.scale_factor();
                    let logical_size = size.to_logical::<f64>(scale_factor);
                    self.resize_webviews(logical_size.width, logical_size.height);
                }
            }
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: BrowserEvent) {
        match event {
            BrowserEvent::Navigate(url) => {
                println!("Navigating to: {}", url);
                if let Some(webview) = &self.content_webview {
                    let url = if !url.contains("://") {
                        format!("https://{}", url)
                    } else {
                        url
                    };
                    
                    // Update current URL and toolbar
                    self.current_url = url.clone();
                    self.update_toolbar_url(&url);
                    
                    let _ = webview.load_url(url.as_str());
                }
            }
            BrowserEvent::Settings => {
                println!("Settings requested!");
                if let Some(webview) = &self.content_webview {
                    let html = include_str!("settings.html");
                    let _ = webview.load_html(html);
                    
                    // Update toolbar to show settings page
                    self.current_url = "about:settings".to_string();
                    self.update_toolbar_url(&self.current_url);
                }
            }
        }
    }
}

pub fn run_browser() -> i32 {
    let event_loop = EventLoop::<BrowserEvent>::with_user_event()
        .build()
        .expect("Failed to create event loop");
    
    let proxy = event_loop.create_proxy();
    let mut app = App::new(proxy);
    
    event_loop
        .run_app(&mut app)
        .expect("Failed to run application");
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_web_main() -> i32 {
    run_browser()
}