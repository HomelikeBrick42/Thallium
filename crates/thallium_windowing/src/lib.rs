#![doc = include_str!("../README.md")]

use thallium_ecs::{App, ResMut, Resource};

use thiserror::Error;
#[doc(no_inline)]
pub use winit;
use winit::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::Window,
};

/// The resource for getting the window size
/// Because users of the crate cannot mutate this, it should always be requested using [`Res`](thallium_ecs::Res) in system parameters
pub struct WindowSize {
    pub(crate) width: usize,
    pub(crate) height: usize,
}

impl WindowSize {
    /// Returns the width of the window
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns the height of the window
    pub fn height(&self) -> usize {
        self.height
    }
}

impl Resource for WindowSize {}

/// The error returned from [`run_window`]
#[derive(Debug, Error)]
pub enum RunWindowError {
    /// [`winit::error::EventLoopError`]
    #[error("{0}")]
    EventLoopError(#[from] winit::error::EventLoopError),
    /// [`winit::error::OsError`]
    #[error("{0}")]
    OsError(#[from] winit::error::OsError),
}

/// Creates the window specified by the builder, then runs the event loop
/// Check the docs for [`EventLoop::run`] for whether this function will ever return
pub fn run_window(
    app: &mut App,
    window_builder: winit::window::WindowBuilder,
    mut on_update: impl FnMut(&mut App),
    mut on_render: impl FnMut(&mut App),
    on_quit: impl FnOnce(&mut App),
) -> Result<(), RunWindowError> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let window = window_builder.with_visible(false).build(&event_loop)?;

    fn event_handler(
        event: Event<()>,
        elwt: &EventLoopWindowTarget<()>,
        window: &Window,
        app: &mut App,
        on_update: &mut impl FnMut(&mut App),
        on_render: &mut impl FnMut(&mut App),
        on_quit: &mut Option<impl FnOnce(&mut App)>,
    ) -> Result<(), RunWindowError> {
        match event {
            Event::NewEvents(StartCause::Init) => {
                window.set_visible(true);
            }
            Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
                WindowEvent::Resized(new_size) => {
                    app.run(|mut window_size: ResMut<'_, WindowSize>| {
                        window_size.width = new_size.width as _;
                        window_size.height = new_size.height as _;
                    });
                }
                WindowEvent::CloseRequested => {
                    elwt.exit();
                }
                WindowEvent::Destroyed => {
                    elwt.exit();
                }
                WindowEvent::RedrawRequested => on_render(app),
                _ => {}
            },
            Event::AboutToWait => {
                on_update(app);
                window.request_redraw();
            }
            Event::LoopExiting => {
                on_quit.take().unwrap()(app);
                window.set_visible(false);
            }
            _ => {}
        }
        Ok(())
    }

    let mut error = Ok(());
    event_loop.run({
        let error = &mut error;

        let on_update = &mut on_update;
        let on_render = &mut on_render;
        let mut on_quit = Some(on_quit);

        move |event, elwt| {
            *error = std::mem::replace(error, Ok(())).or(event_handler(
                event,
                elwt,
                &window,
                app,
                on_update,
                on_render,
                &mut on_quit,
            ));
            if error.is_err() {
                elwt.exit();
            }
        }
    })?;
    error
}