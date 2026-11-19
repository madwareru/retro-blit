use std::{convert::TryInto, time::Duration};

use super::{ContextHandler, KeyCode, KeyMods, RetroBlitContext};

/// Deterministic, windowless driver for testing a [`ContextHandler`].
///
/// Methods execute immediately in ordinary Rust control flow. Each `after`
/// argument first advances the timeline relative to the previous action and
/// runs intervening fixed-step updates. Zero-delay actions remain grouped
/// before the same pending update. Snapshots are optional and do not consume
/// the runner. No window, GPU, audio, or egui rendering is involved.
///
/// # Example
///
/// ```rust,ignore
/// let mut test = HeadlessRunner::new(MyGame::new());
/// test.mouse_move(Duration::ZERO, 10.0, 20.0);
/// test.mouse_click(Duration::from_millis(50), 0);
///
/// for index in 0..4 {
///     let (x, y) = test.inspect(Duration::ZERO, |game| {
///         game.buttons[index].position()
///     });
///     test.mouse_move(Duration::ZERO, x, y);
///     test.mouse_click(Duration::ZERO, 0);
/// }
///
/// let snapshot = test.snapshot();
/// ```
pub struct HeadlessRunner<H: ContextHandler> {
    handler: H,
    context: RetroBlitContext,
    current_time: Duration,
    next_update_time: Duration,
    pending_update: bool,
    frame_dt: Duration,
    press_duration: Duration,
}

impl<H: ContextHandler> HeadlessRunner<H> {
    /// Creates a runner and immediately calls [`ContextHandler::init`].
    ///
    /// Updates use an 8 ms timestep and convenience presses last 30 ms by
    /// default.
    pub fn new(mut handler: H) -> Self {
        let mut context = RetroBlitContext::new(&handler);
        handler.init(&mut context);
        Self {
            handler,
            context,
            current_time: Duration::ZERO,
            next_update_time: Duration::ZERO,
            pending_update: false,
            frame_dt: Duration::from_millis(8),
            press_duration: Duration::from_millis(30),
        }
    }

    /// Sets the fixed interval passed to [`ContextHandler::update`].
    ///
    /// This controls update frequency, not action timestamps. The default is
    /// 8 ms.
    ///
    /// # Panics
    ///
    /// Panics if `frame_dt` is zero.
    pub fn with_frame_dt(&mut self, frame_dt: Duration) {
        assert!(!frame_dt.is_zero(), "frame duration must be nonzero");
        self.frame_dt = frame_dt;
    }

    /// Sets the down-to-up delay for subsequent clicks and key presses.
    ///
    /// The default is 30 ms. Durations shorter than the update interval are
    /// allowed, so an update is not guaranteed to observe every held input.
    pub fn with_press_duration(&mut self, press_duration: Duration) {
        self.press_duration = press_duration;
    }

    /// Advances the timeline to `exact_time` without adding an event.
    ///
    /// This acts as a barrier before the next action. It does nothing if the
    /// timeline has already reached `exact_time`.
    pub fn wait_until(&mut self, exact_time: Duration) {
        self.current_time = self.current_time.max(exact_time);
    }

    /// Executes a move to fixed game-buffer coordinates.
    pub fn mouse_move(&mut self, after: Duration, x: f32, y: f32) {
        self.prepare(after);
        self.context.mouse_x = x;
        self.context.mouse_y = y;
    }

    /// Executes [`ContextHandler::on_mouse_down`] for `button_number`.
    pub fn mouse_down(&mut self, after: Duration, button_number: u8) {
        self.prepare(after);
        self.handler.on_mouse_down(&mut self.context, button_number);
    }

    /// Executes [`ContextHandler::on_mouse_up`] for `button_number`.
    pub fn mouse_up(&mut self, after: Duration, button_number: u8) {
        self.prepare(after);
        self.handler.on_mouse_up(&mut self.context, button_number);
    }

    /// Executes a mouse down/up pair separated by the current press duration.
    ///
    /// Subsequent relative delays are measured from the release.
    pub fn mouse_click(&mut self, after: Duration, button_number: u8) {
        self.mouse_down(after, button_number);
        let press_duration = self.press_duration;
        self.mouse_up(press_duration, button_number);
    }

    /// Executes a key press, updating held-key and modifier state before the callback.
    pub fn key_down(&mut self, after: Duration, key_code: KeyCode) {
        self.prepare(after);
        self.context.keys_pressed.insert(key_code);
        let key_mods = self.key_mods();
        self.context.key_mods_pressed = key_mods;
        self.handler
            .on_key_down(&mut self.context, key_code, key_mods);
    }

    /// Executes a key release, updating held-key and modifier state before the callback.
    pub fn key_up(&mut self, after: Duration, key_code: KeyCode) {
        self.prepare(after);
        self.context.keys_pressed.remove(&key_code);
        let key_mods = self.key_mods();
        self.context.key_mods_pressed = key_mods;
        self.handler
            .on_key_up(&mut self.context, key_code, key_mods);
    }

    /// Executes a key down/up pair separated by the current press duration.
    ///
    /// Subsequent relative delays are measured from the release.
    pub fn key_press(&mut self, after: Duration, key_code: KeyCode) {
        self.key_down(after, key_code);
        let press_duration = self.press_duration;
        self.key_up(press_duration, key_code);
    }

    /// Executes a read-only inspection of the current handler state.
    ///
    /// Use this to assert internal game state or read coordinates and other
    /// values for later actions. The closure's return value is returned
    /// directly, so inspections work naturally inside loops and conditionals.
    pub fn inspect<T>(&mut self, after: Duration, f: impl FnOnce(&H) -> T) -> T {
        self.prepare(after);
        f(&self.handler)
    }

    /// Returns the current native-resolution RGBA buffer.
    ///
    /// If the latest action has a pending fixed-step update, that update runs
    /// before the snapshot. The runner remains usable for more actions.
    pub fn snapshot(&mut self) -> HeadlessSnapshot {
        if self.pending_update {
            self.update();
            self.pending_update = false;
        }
        HeadlessSnapshot::from_context(&self.context)
    }

    fn prepare(&mut self, after: Duration) {
        self.current_time += after;
        while self.next_update_time < self.current_time {
            self.update();
        }
        self.pending_update = true;
    }

    fn update(&mut self) {
        self.handler
            .update(&mut self.context, self.frame_dt.as_secs_f32());
        self.next_update_time += self.frame_dt;
    }

    fn key_mods(&self) -> KeyMods {
        let pressed = &self.context.keys_pressed;
        KeyMods {
            shift: pressed.contains(&KeyCode::LeftShift) || pressed.contains(&KeyCode::RightShift),
            control: pressed.contains(&KeyCode::LeftControl)
                || pressed.contains(&KeyCode::RightControl),
            option: pressed.contains(&KeyCode::LeftAlt) || pressed.contains(&KeyCode::RightAlt),
            command: pressed.contains(&KeyCode::LeftSuper)
                || pressed.contains(&KeyCode::RightSuper),
        }
    }
}

pub struct HeadlessSnapshot {
    width: usize,
    height: usize,
    rgba: Vec<u8>,
}

impl HeadlessSnapshot {
    fn from_context(context: &RetroBlitContext) -> Self {
        let mut rgba = Vec::with_capacity(context.buffer_pixels.len() * 4);
        for &color in &context.buffer_pixels {
            rgba.extend_from_slice(&context.get_palette(color));
            rgba.push(255);
        }
        Self {
            width: context.buffer_width,
            height: context.buffer_height,
            rgba,
        }
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn as_rgba(&self) -> &[u8] {
        &self.rgba
    }

    pub fn rgba_pixel(&self, x: usize, y: usize) -> [u8; 4] {
        let offset = (y * self.width + x) * 4;
        self.rgba[offset..offset + 4].try_into().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::super::{ContextHandler, KeyCode, RetroBlitContext, WindowMode};
    use super::{HeadlessRunner, HeadlessSnapshot};

    struct RecordingHandler {
        log: Vec<String>,
        updates: usize,
    }

    impl RecordingHandler {
        fn new() -> Self {
            Self {
                log: Vec::new(),
                updates: 0,
            }
        }
    }

    impl ContextHandler for RecordingHandler {
        fn get_window_title(&self) -> &'static str {
            "headless recording test"
        }

        fn get_window_mode(&self) -> WindowMode {
            WindowMode::Mode64x64
        }

        fn on_mouse_down(&mut self, ctx: &mut RetroBlitContext, button_number: u8) {
            let (x, y) = ctx.get_mouse_pos();
            self.log
                .push(format!("mouse_down:{}:{}:{}", button_number, x, y));
        }

        fn on_mouse_up(&mut self, _ctx: &mut RetroBlitContext, button_number: u8) {
            self.log.push(format!("mouse_up:{}", button_number));
        }

        fn on_key_down(
            &mut self,
            ctx: &mut RetroBlitContext,
            key_code: KeyCode,
            key_mods: super::super::KeyMods,
        ) {
            self.log.push(format!(
                "key_down:{}:{}:{}",
                key_code as u8,
                key_mods.shift,
                ctx.is_key_pressed(key_code)
            ));
        }

        fn on_key_up(
            &mut self,
            ctx: &mut RetroBlitContext,
            key_code: KeyCode,
            key_mods: super::super::KeyMods,
        ) {
            self.log.push(format!(
                "key_up:{}:{}:{}",
                key_code as u8,
                key_mods.shift,
                ctx.is_key_pressed(key_code)
            ));
        }

        fn init(&mut self, ctx: &mut RetroBlitContext) {
            self.log.push("init".to_owned());
            ctx.set_palette(1, [10, 20, 30]);
            ctx.set_palette(2, [40, 50, 60]);
            ctx.set_palette(3, [70, 80, 90]);
            ctx.put_pixel(0, 0, 1);
        }

        fn update(&mut self, ctx: &mut RetroBlitContext, dt: f32) {
            self.updates += 1;
            self.log.push(format!(
                "update:{:.3}:{}:{}",
                dt,
                ctx.is_key_pressed(KeyCode::A),
                ctx.is_key_mod_pressed(super::super::KeyMod::Shift)
            ));
            ctx.clear(0);
            ctx.put_pixel(2, 3, self.updates as u8);
        }
    }

    #[test]
    fn snapshot_uses_native_dimensions_and_final_palette() {
        let mut context = RetroBlitContext::new(&RecordingHandler::new());
        context.set_palette(1, [10, 20, 30]);
        context.put_pixel(2, 3, 1);
        let snapshot = HeadlessSnapshot::from_context(&context);
        assert_eq!(snapshot.dimensions(), (64, 64));
        assert_eq!(snapshot.rgba_pixel(2, 3), [10, 20, 30, 255]);
        assert_eq!(snapshot.as_rgba().len(), 64 * 64 * 4);
    }

    #[test]
    fn runner_dispatches_events_in_call_order() {
        let delay = Duration::from_millis(10);
        let mut runner = HeadlessRunner::new(RecordingHandler::new());
        runner.with_frame_dt(delay);
        runner.mouse_move(Duration::ZERO, 10.0, 20.0);
        runner.key_down(delay, KeyCode::A);
        runner.key_down(Duration::ZERO, KeyCode::LeftShift);
        runner.mouse_down(Duration::ZERO, 0);
        runner.key_up(Duration::ZERO, KeyCode::LeftShift);
        runner.snapshot();

        let log = runner.inspect(Duration::ZERO, |handler| handler.log.clone());
        assert_eq!(
            log.as_slice(),
            [
                "init",
                "update:0.010:false:false",
                &format!("key_down:{}:false:true", KeyCode::A as u8),
                &format!("key_down:{}:true:true", KeyCode::LeftShift as u8),
                "mouse_down:0:10:20",
                &format!("key_up:{}:false:false", KeyCode::LeftShift as u8),
                "update:0.010:true:false",
            ]
        );
    }

    #[test]
    fn inspect_returns_handler_values_for_mouse_moves() {
        let mut runner = HeadlessRunner::new(RecordingHandler::new());
        runner.with_frame_dt(Duration::from_millis(10));
        let (x, y) = runner.inspect(Duration::from_millis(15), |handler| {
            (handler.updates as f32, 20.0)
        });
        runner.mouse_move(Duration::ZERO, x, y);
        runner.mouse_down(Duration::ZERO, 0);
        runner.snapshot();

        let log = runner.inspect(Duration::ZERO, |handler| handler.log.clone());
        assert_eq!(
            log.as_slice(),
            [
                "init",
                "update:0.010:false:false",
                "update:0.010:false:false",
                "mouse_down:0:2:20",
                "update:0.010:false:false",
            ]
        );
    }

    #[test]
    fn runner_uses_fixed_steps_and_snapshots_after_last_event() {
        let mut runner = HeadlessRunner::new(RecordingHandler::new());
        runner.with_frame_dt(Duration::from_millis(10));
        runner.mouse_move(Duration::from_millis(5), 10.0, 20.0);
        runner.mouse_down(Duration::from_millis(10), 0);
        let snapshot = runner.snapshot();

        let log = runner.inspect(Duration::ZERO, |handler| handler.log.clone());
        assert_eq!(
            log.as_slice(),
            [
                "init",
                "update:0.010:false:false",
                "update:0.010:false:false",
                "mouse_down:0:10:20",
                "update:0.010:false:false",
            ]
        );
        assert_eq!(snapshot.rgba_pixel(2, 3), [70, 80, 90, 255]);
    }

    #[test]
    fn mouse_click_uses_custom_duration_and_continues_after_release() {
        let mut runner = HeadlessRunner::new(RecordingHandler::new());
        runner.with_frame_dt(Duration::from_millis(10));
        runner.with_press_duration(Duration::from_millis(20));
        runner.mouse_click(Duration::ZERO, 0);
        let updates = runner.inspect(Duration::from_millis(5), |handler| handler.updates);
        assert_eq!(updates, 3);
        runner.snapshot();

        let log = runner.inspect(Duration::ZERO, |handler| handler.log.clone());
        assert_eq!(
            log.as_slice(),
            [
                "init",
                "mouse_down:0:0:0",
                "update:0.010:false:false",
                "update:0.010:false:false",
                "mouse_up:0",
                "update:0.010:false:false",
                "update:0.010:false:false",
            ]
        );
    }

    #[test]
    fn key_press_uses_default_30ms_duration() {
        let mut runner = HeadlessRunner::new(RecordingHandler::new());
        runner.with_frame_dt(Duration::from_millis(10));
        runner.key_press(Duration::ZERO, KeyCode::A);
        runner.snapshot();

        let log = runner.inspect(Duration::ZERO, |handler| handler.log.clone());
        assert_eq!(
            log.as_slice(),
            [
                "init",
                &format!("key_down:{}:false:true", KeyCode::A as u8),
                "update:0.010:true:false",
                "update:0.010:true:false",
                "update:0.010:true:false",
                &format!("key_up:{}:false:false", KeyCode::A as u8),
                "update:0.010:false:false",
            ]
        );
    }

    #[test]
    fn short_key_press_can_release_before_the_same_update() {
        let mut runner = HeadlessRunner::new(RecordingHandler::new());
        runner.with_frame_dt(Duration::from_millis(20));
        runner.with_press_duration(Duration::from_millis(5));
        runner.key_press(Duration::from_millis(5), KeyCode::A);
        runner.snapshot();

        let log = runner.inspect(Duration::ZERO, |handler| handler.log.clone());
        assert_eq!(
            log.as_slice(),
            [
                "init",
                "update:0.020:false:false",
                &format!("key_down:{}:false:true", KeyCode::A as u8),
                &format!("key_up:{}:false:false", KeyCode::A as u8),
                "update:0.020:false:false",
            ]
        );
    }

    #[test]
    fn runner_inspects_internal_state_at_current_time() {
        let mut runner = HeadlessRunner::new(RecordingHandler::new());
        runner.with_frame_dt(Duration::from_millis(10));
        let initial_updates = runner.inspect(Duration::ZERO, |handler| handler.updates);
        assert_eq!(initial_updates, 0);
        let later_updates = runner.inspect(Duration::from_millis(15), |handler| handler.updates);
        assert_eq!(later_updates, 2);
        let snapshot = runner.snapshot();

        let log = runner.inspect(Duration::ZERO, |handler| handler.log.clone());
        assert_eq!(
            log.as_slice(),
            [
                "init",
                "update:0.010:false:false",
                "update:0.010:false:false",
                "update:0.010:false:false",
            ]
        );
        assert_eq!(snapshot.rgba_pixel(2, 3), [70, 80, 90, 255]);
    }

    #[test]
    fn empty_runner_returns_post_init_snapshot() {
        let mut runner = HeadlessRunner::new(RecordingHandler::new());
        let snapshot = runner.snapshot();
        let log = runner.inspect(Duration::ZERO, |handler| handler.log.clone());
        assert_eq!(log.as_slice(), ["init"]);
        assert_eq!(snapshot.rgba_pixel(0, 0), [10, 20, 30, 255]);
    }

    #[test]
    fn default_frame_dt_is_8ms() {
        let mut runner = HeadlessRunner::new(RecordingHandler::new());
        runner.inspect(Duration::ZERO, |_handler| {});
        runner.snapshot();
        runner.snapshot();
        let log = runner.inspect(Duration::ZERO, |handler| handler.log.clone());
        assert_eq!(log.as_slice(), ["init", "update:0.008:false:false"]);
    }

    #[test]
    fn wait_until_delays_only_the_next_event() {
        let mut runner = HeadlessRunner::new(RecordingHandler::new());
        runner.with_frame_dt(Duration::from_millis(10));
        runner.mouse_move(Duration::ZERO, 10.0, 20.0);
        runner.wait_until(Duration::from_millis(25));
        runner.wait_until(Duration::from_millis(10));
        runner.mouse_down(Duration::ZERO, 0);
        runner.snapshot();

        let log = runner.inspect(Duration::ZERO, |handler| handler.log.clone());
        assert_eq!(
            log.as_slice(),
            [
                "init",
                "update:0.010:false:false",
                "update:0.010:false:false",
                "update:0.010:false:false",
                "mouse_down:0:10:20",
                "update:0.010:false:false",
            ]
        );
    }

    #[test]
    #[should_panic]
    fn zero_frame_dt_is_rejected() {
        let mut runner = HeadlessRunner::new(RecordingHandler::new());
        runner.with_frame_dt(Duration::ZERO);
    }
}
