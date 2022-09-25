use retro_blit::rendering::BlittableSurface;
use retro_blit::window::{ContextHandler, RetroBlitContext, WindowMode};
use crate::state::ToolsAppState;
use crate::ui_view::ToolsAppStateView;

pub mod toolbar;
pub mod state;
pub mod map_state;
pub mod ui_view;

const TOOLBARS_GRAPHICS: &[u8] = include_bytes!("map_editor_toolbars.im256");

pub struct EditorApp {
    palette: Vec<[u8; 3]>,
    toolbars_graphics: BlittableSurface,
    tools_app_state: ToolsAppState,
    tools_app_state_view: ToolsAppStateView
}
impl EditorApp {
    pub fn new() -> Self {
        let (palette, toolbars_graphics) = retro_blit
            ::format_loaders
            ::im_256
            ::Image
            ::load_from(TOOLBARS_GRAPHICS).unwrap();

        Self {
            palette,
            toolbars_graphics,
            tools_app_state: ToolsAppState::default(),
            tools_app_state_view: ToolsAppStateView::make()
        }
    }
}
impl ContextHandler for EditorApp {
    fn get_window_title(&self) -> &'static str {
        "map editor"
    }

    fn get_window_mode(&self) -> WindowMode {
        WindowMode::ModeXFrameless
    }

    fn on_mouse_down(&mut self, _ctx: &mut RetroBlitContext, _button_number: u8) {
        self.tools_app_state_view.on_button_down();
    }

    fn on_mouse_up(&mut self, _ctx: &mut RetroBlitContext, _button_number: u8) {
        self.tools_app_state_view.on_button_up();
    }

    fn init(&mut self, ctx: &mut RetroBlitContext) {
        for i in 0..self.palette.len() {
            ctx.set_palette(i as _, self.palette[i]);
        }
    }

    fn update(&mut self, ctx: &mut RetroBlitContext, _dt: f32) {
        let tools_app_state_view_ref = &mut self.tools_app_state_view;
        let surface_ref = &self.toolbars_graphics;
        self.tools_app_state = tools_app_state_view_ref.update(
            ctx.get_mouse_pos(),
            surface_ref,
            self.tools_app_state
        );
        ctx.clear(26);
        tools_app_state_view_ref.draw(surface_ref, ctx);
    }
}

fn main() {
    retro_blit::window::start(EditorApp::new());
}
