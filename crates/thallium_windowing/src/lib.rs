#![doc = include_str!("../README.md")]

use std::{collections::HashMap, sync::Arc};
use thallium_ecs::{App, CurrentTick, ResMut, Resource};
use thiserror::Error;
use winit::{
    event::{ElementState, Event, StartCause, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    keyboard::KeyCode,
};

#[doc(no_inline)]
pub use winit;

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

/// The [`Resource`] for getting access to the keyboard
///
/// Because users of the crate cannot mutate this, it should always be requested using [`Res`](thallium_ecs::Res) in system parameters
pub struct Keyboard {
    keys: HashMap<KeyCode, KeyState>,
    current_tick: u64,
}

struct KeyState {
    pressed: bool,
    last_changed_tick: u64,
}

impl Keyboard {
    /// Returns whether `key` is currently held down
    pub fn key_down(&self, key: KeyCode) -> bool {
        self.keys.get(&key).map_or(false, |state| state.pressed)
    }

    /// Returns whether `key` is not currently held down
    pub fn key_up(&self, key: KeyCode) -> bool {
        self.keys.get(&key).map_or(true, |state| !state.pressed)
    }

    /// Returns whether `key` was pressed this tick
    pub fn key_pressed(&self, key: KeyCode) -> bool {
        self.keys.get(&key).map_or(false, |state| {
            state.pressed && self.current_tick == state.last_changed_tick
        })
    }

    /// Returns whether `key` was released this tick
    pub fn key_released(&self, key: KeyCode) -> bool {
        self.keys.get(&key).map_or(false, |state| {
            !state.pressed && self.current_tick == state.last_changed_tick
        })
    }

    /// Returns whether `key` was pressed since `tick`
    pub fn key_pressed_since(&self, key: KeyCode, tick: u64) -> bool {
        self.keys.get(&key).map_or(false, |state| {
            state.pressed && tick > state.last_changed_tick
        })
    }

    /// Returns whether `key` was released since `tick`
    pub fn key_released_since(&self, key: KeyCode, tick: u64) -> bool {
        self.keys.get(&key).map_or(false, |state| {
            !state.pressed && tick > state.last_changed_tick
        })
    }
}

impl Resource for Keyboard {}

/// Gives access to the winit window, when accepting this type in system parameters be aware that the system may not be running on the main thread
/// Accepting the other `Window*` types into your systems is always preferred
pub struct Window {
    /// The [`winit::Window`](winit::window::Window)
    pub window: Arc<winit::window::Window>,
}

impl Resource for Window {}

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
/// This function should be called on the main thread
///
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

    let window = Arc::new(window_builder.with_visible(false).build(&event_loop)?);
    app.add_resource(Window {
        window: window.clone(),
    });
    app.add_resource(WindowSize {
        width: window.inner_size().width as _,
        height: window.inner_size().height as _,
    });

    fn event_handler(
        event: Event<()>,
        elwt: &EventLoopWindowTarget<()>,
        window: &winit::window::Window,
        app: &mut App,
        mut on_update: impl FnMut(&mut App),
        mut on_render: impl FnMut(&mut App),
        on_quit: &mut Option<impl FnOnce(&mut App)>,
    ) -> Result<(), RunWindowError> {
        match event {
            Event::NewEvents(StartCause::Init) => {
                window.set_visible(true);
            }
            Event::NewEvents(StartCause::Poll) => {
                app.next_tick();
            }
            Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
                WindowEvent::Resized(new_size) => {
                    app.run(|mut window_size: ResMut<'_, WindowSize>| {
                        window_size.width = new_size.width as _;
                        window_size.height = new_size.height as _;
                    });
                }
                WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                    elwt.exit();
                }
                WindowEvent::KeyboardInput {
                    device_id: _,
                    event,
                    is_synthetic: _,
                } => match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(key_code) => app.run(
                        |mut keyboard: ResMut<'_, Keyboard>,
                         CurrentTick(current_tick): CurrentTick| {
                            keyboard.keys.insert(
                                key_code,
                                KeyState {
                                    pressed: matches!(event.state, ElementState::Pressed),
                                    last_changed_tick: current_tick,
                                },
                            );
                        },
                    ),
                    winit::keyboard::PhysicalKey::Unidentified(_) => {}
                },
                WindowEvent::RedrawRequested => on_render(app),
                _ => {}
            },
            Event::AboutToWait => {
                app.run(
                    |mut keyboard: ResMut<'_, Keyboard>, CurrentTick(current_tick): CurrentTick| {
                        keyboard.current_tick = current_tick;
                    },
                );
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
                &mut *on_update,
                &mut *on_render,
                &mut on_quit,
            ));
            if error.is_err() {
                elwt.exit();
            }
        }
    })?;
    error
}
