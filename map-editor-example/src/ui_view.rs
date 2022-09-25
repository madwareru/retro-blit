use std::marker::PhantomData;
use retro_blit::{
    rendering::blittable::Rect,
    rendering::BlittableSurface,
    window::RetroBlitContext
};
use crate::{
    toolbar::{Toolbar, ToolbarKind},
    state::{Tool, BrushSize, DrawMode, TerrainTile, TerrainToolState}
};
use crate::state::{ToolsAppState, BuildingKind, BuildingToolState, MountainToolState, NatureKind, NatureToolState, PropKind, PropToolState, RoadToolState, UnitKind, UnitToolState};

pub trait UiView<TModel: Copy> {
    fn init(&mut self, model: TModel);
    fn on_button_down(&mut self);
    fn on_button_up(&mut self);
    fn update(&mut self, mouse_pos: (f32, f32), surface: &BlittableSurface) -> TModel;
    fn draw(&self, surface: &BlittableSurface, dest: &mut RetroBlitContext);
}

pub struct TypedToolbarView<TData: Copy + TryFrom<u8> + Into<u8>> {
    toolbar: Toolbar,
    _fantom_data: PhantomData<TData>
}

impl<TData: Copy + TryFrom<u8> + Into<u8>> TypedToolbarView<TData> {
    pub(crate) fn get_selection(&self) -> Option<TData> {
        self.toolbar.get_selection().and_then(|it| it.try_into().ok())
    }
}

impl <TData: Copy + TryFrom<u8> + Into<u8>> TypedToolbarView<TData> {
    pub fn make(
        x: usize,
        y: usize,
        rect: Rect,
        kind: ToolbarKind,
        default_value: Option<TData>
    ) -> Self {
        let mut toolbar = Toolbar::make(x, y, rect, kind);
        toolbar.set_selection(default_value.map(|it| it.into()));
        Self { toolbar, _fantom_data: PhantomData }
    }
}
impl<TData: Copy + TryFrom<u8> + Into<u8>> UiView<Option<TData>> for TypedToolbarView<TData> {
    fn init(&mut self, model: Option<TData>) {
        self.toolbar.set_selection(model.map(|it| it.into()))
    }

    fn on_button_down(&mut self) {
        self.toolbar.on_button_down();
    }

    fn on_button_up(&mut self) {
        self.toolbar.on_button_up();
    }

    fn update(&mut self, mouse_pos: (f32, f32), surface: &BlittableSurface) -> Option<TData> {
        self.toolbar.update(mouse_pos, surface);
        self.toolbar.get_selection().and_then(|it| it.try_into().ok())
    }

    fn draw(&self, surface: &BlittableSurface, dest: &mut RetroBlitContext) {
        self.toolbar.draw(surface, dest)
    }
}

pub struct TerrainToolSubView {
    terrain_tile_toolbar: TypedToolbarView<TerrainTile>,
    brush_size_toolbar: TypedToolbarView<BrushSize>
}
impl TerrainToolSubView {
    pub fn make() -> Self {
        let default_state = TerrainToolState::default();
        let terrain_tile_toolbar = TypedToolbarView::make(
            0, 24,
            Rect {
                x_range: 164..164+115,
                y_range: 5..5+105
            },
            ToolbarKind::Vertical,
            default_state.tile
        );
        let brush_size_toolbar = TypedToolbarView::make(
            129, 223,
            Rect {
                x_range: 61..61+62,
                y_range: 200..200+85
            },
            ToolbarKind::Horizontal,
            default_state.brush_size
        );
        Self {
            terrain_tile_toolbar,
            brush_size_toolbar
        }
    }
}
impl UiView<TerrainToolState> for TerrainToolSubView {
    fn init(&mut self, model: TerrainToolState) {
        self.terrain_tile_toolbar.init(model.tile);
        self.brush_size_toolbar.init(model.brush_size);
    }

    fn on_button_down(&mut self) {
        self.terrain_tile_toolbar.on_button_down();
        self.brush_size_toolbar.on_button_down();
    }

    fn on_button_up(&mut self) {
        self.terrain_tile_toolbar.on_button_up();
        self.brush_size_toolbar.on_button_up();
    }

    fn update(&mut self, mouse_pos: (f32, f32), surface: &BlittableSurface) -> TerrainToolState {
        TerrainToolState {
            tile: self.terrain_tile_toolbar.update(mouse_pos, surface),
            brush_size: self.brush_size_toolbar.update(mouse_pos, surface)
        }
    }

    fn draw(&self, surface: &BlittableSurface, dest: &mut RetroBlitContext) {
        self.terrain_tile_toolbar.draw(surface, dest);
        self.brush_size_toolbar.draw(surface, dest);
    }
}

pub struct NatureToolSubView {
    nature_kind_toolbar: TypedToolbarView<NatureKind>,
    draw_mode_toolbar: TypedToolbarView<DrawMode>,
    brush_size_toolbar: TypedToolbarView<BrushSize>
}
impl NatureToolSubView {
    pub fn make() -> Self {
        let default_state = NatureToolState::default();
        let nature_kind_toolbar = TypedToolbarView::make(
            0, 24,
            Rect {
                x_range: 28..28+115,
                y_range: 129..129+65
            },
            ToolbarKind::Vertical,
            default_state.nature_kind
        );
        let draw_mode_toolbar = TypedToolbarView::make(
            297, 24,
            Rect {
                x_range: 164..164+115,
                y_range: 163..163+65
            },
            ToolbarKind::Vertical,
            default_state.draw_mode
        );
        let brush_size_toolbar = TypedToolbarView::make(
            129, 223,
            Rect {
                x_range: 61..61+62,
                y_range: 200..200+85
            },
            ToolbarKind::Horizontal,
            default_state.brush_size
        );
        Self {
            nature_kind_toolbar,
            draw_mode_toolbar,
            brush_size_toolbar
        }
    }
}
impl UiView<NatureToolState> for NatureToolSubView {
    fn init(&mut self, model: NatureToolState) {
        self.nature_kind_toolbar.init(model.nature_kind);
        self.draw_mode_toolbar.init(model.draw_mode);
        self.brush_size_toolbar.init(model.brush_size);
    }

    fn on_button_down(&mut self) {
        self.nature_kind_toolbar.on_button_down();
        self.draw_mode_toolbar.on_button_down();
        self.brush_size_toolbar.on_button_down();
    }

    fn on_button_up(&mut self) {
        self.nature_kind_toolbar.on_button_up();
        self.draw_mode_toolbar.on_button_up();
        self.brush_size_toolbar.on_button_up();
    }

    fn update(&mut self, mouse_pos: (f32, f32), surface: &BlittableSurface) -> NatureToolState {
        NatureToolState {
            nature_kind: self.nature_kind_toolbar.update(mouse_pos, surface),
            draw_mode: self.draw_mode_toolbar.update(mouse_pos, surface),
            brush_size: self.brush_size_toolbar.update(mouse_pos, surface)
        }
    }

    fn draw(&self, surface: &BlittableSurface, dest: &mut RetroBlitContext) {
        self.nature_kind_toolbar.draw(surface, dest);
        self.draw_mode_toolbar.draw(surface, dest);
        self.brush_size_toolbar.draw(surface, dest);
    }
}

pub struct MountainToolSubView {
    draw_mode_toolbar: TypedToolbarView<DrawMode>,
    brush_size_toolbar: TypedToolbarView<BrushSize>
}
impl MountainToolSubView {
    pub fn make() -> Self {
        let default_state = MountainToolState::default();
        let draw_mode_toolbar = TypedToolbarView::make(
            297, 24,
            Rect {
                x_range: 164..164+115,
                y_range: 163..163+65
            },
            ToolbarKind::Vertical,
            default_state.draw_mode
        );
        let brush_size_toolbar = TypedToolbarView::make(
            129, 223,
            Rect {
                x_range: 61..61+62,
                y_range: 200..200+85
            },
            ToolbarKind::Horizontal,
            default_state.brush_size
        );
        Self {
            draw_mode_toolbar,
            brush_size_toolbar
        }
    }
}
impl UiView<MountainToolState> for MountainToolSubView {
    fn init(&mut self, model: MountainToolState) {
        self.draw_mode_toolbar.init(model.draw_mode);
        self.brush_size_toolbar.init(model.brush_size);
    }

    fn on_button_down(&mut self) {
        self.draw_mode_toolbar.on_button_down();
        self.brush_size_toolbar.on_button_down();
    }

    fn on_button_up(&mut self) {
        self.draw_mode_toolbar.on_button_up();
        self.brush_size_toolbar.on_button_up();
    }

    fn update(&mut self, mouse_pos: (f32, f32), surface: &BlittableSurface) -> MountainToolState {
        MountainToolState {
            draw_mode: self.draw_mode_toolbar.update(mouse_pos, surface),
            brush_size: self.brush_size_toolbar.update(mouse_pos, surface)
        }
    }

    fn draw(&self, surface: &BlittableSurface, dest: &mut RetroBlitContext) {
        self.draw_mode_toolbar.draw(surface, dest);
        self.brush_size_toolbar.draw(surface, dest);
    }
}

pub struct PropToolSubView {
    prop_kind_toolbar: TypedToolbarView<PropKind>,
    draw_mode_toolbar: TypedToolbarView<DrawMode>
}
impl PropToolSubView {
    pub fn make() -> Self {
        let default_state = PropToolState::default();
        let prop_kind_toolbar = TypedToolbarView::make(
            0, 24,
            Rect {
                x_range: 284..284+115,
                y_range: 179..179+128
            },
            ToolbarKind::Vertical,
            default_state.prop_kind
        );
        let draw_mode_toolbar = TypedToolbarView::make(
            297, 24,
            Rect {
                x_range: 164..164+115,
                y_range: 163..163+65
            },
            ToolbarKind::Vertical,
            default_state.draw_mode
        );
        Self {
            prop_kind_toolbar,
            draw_mode_toolbar
        }
    }
}
impl UiView<PropToolState> for PropToolSubView {
    fn init(&mut self, model: PropToolState) {
        self.prop_kind_toolbar.init(model.prop_kind);
        self.draw_mode_toolbar.init(model.draw_mode);
    }

    fn on_button_down(&mut self) {
        self.prop_kind_toolbar.on_button_down();
        self.draw_mode_toolbar.on_button_down();
    }

    fn on_button_up(&mut self) {
        self.prop_kind_toolbar.on_button_up();
        self.draw_mode_toolbar.on_button_up();
    }

    fn update(&mut self, mouse_pos: (f32, f32), surface: &BlittableSurface) -> PropToolState {
        PropToolState {
            prop_kind: self.prop_kind_toolbar.update(mouse_pos, surface),
            draw_mode: self.draw_mode_toolbar.update(mouse_pos, surface)
        }
    }

    fn draw(&self, surface: &BlittableSurface, dest: &mut RetroBlitContext) {
        self.prop_kind_toolbar.draw(surface, dest);
        self.draw_mode_toolbar.draw(surface, dest);
    }
}

pub struct RoadToolSubView {
    draw_mode_toolbar: TypedToolbarView<DrawMode>
}
impl RoadToolSubView {
    pub fn make() -> Self {
        let default_state = RoadToolState::default();
        let draw_mode_toolbar = TypedToolbarView::make(
            297, 24,
            Rect {
                x_range: 164..164+115,
                y_range: 163..163+65
            },
            ToolbarKind::Vertical,
            default_state.draw_mode
        );
        Self {
            draw_mode_toolbar
        }
    }
}

impl UiView<RoadToolState> for RoadToolSubView {
    fn init(&mut self, model: RoadToolState) {
        self.draw_mode_toolbar.init(model.draw_mode);
    }

    fn on_button_down(&mut self) {
        self.draw_mode_toolbar.on_button_down();
    }

    fn on_button_up(&mut self) {
        self.draw_mode_toolbar.on_button_up();
    }

    fn update(&mut self, mouse_pos: (f32, f32), surface: &BlittableSurface) -> RoadToolState {
        RoadToolState {
            draw_mode: self.draw_mode_toolbar.update(mouse_pos, surface)
        }
    }

    fn draw(&self, surface: &BlittableSurface, dest: &mut RetroBlitContext) {
        self.draw_mode_toolbar.draw(surface, dest)
    }
}

pub struct UnitToolSubView {
    unit_kind_toolbar: TypedToolbarView<UnitKind>,
    draw_mode_toolbar: TypedToolbarView<DrawMode>
}
impl UnitToolSubView {
    pub fn make() -> Self {
        let default_state = UnitToolState::default();
        let unit_kind_toolbar = TypedToolbarView::make(
            0, 24,
            Rect {
                x_range: 284..284+115,
                y_range: 5..5+170
            },
            ToolbarKind::Vertical,
            default_state.unit_kind
        );
        let draw_mode_toolbar = TypedToolbarView::make(
            297, 24,
            Rect {
                x_range: 164..164+115,
                y_range: 121..121+107
            },
            ToolbarKind::Vertical,
            default_state.draw_mode
        );
        Self {
            unit_kind_toolbar,
            draw_mode_toolbar
        }
    }
}
impl UiView<UnitToolState> for UnitToolSubView {
    fn init(&mut self, model: UnitToolState) {
        self.unit_kind_toolbar.init(model.unit_kind);
        self.draw_mode_toolbar.init(model.draw_mode);
    }

    fn on_button_down(&mut self) {
        self.unit_kind_toolbar.on_button_down();
        self.draw_mode_toolbar.on_button_down();
    }

    fn on_button_up(&mut self) {
        self.unit_kind_toolbar.on_button_up();
        self.draw_mode_toolbar.on_button_up();
    }

    fn update(&mut self, mouse_pos: (f32, f32), surface: &BlittableSurface) -> UnitToolState {
        UnitToolState {
            unit_kind: self.unit_kind_toolbar.update(mouse_pos, surface),
            draw_mode: self.draw_mode_toolbar.update(mouse_pos, surface)
        }
    }

    fn draw(&self, surface: &BlittableSurface, dest: &mut RetroBlitContext) {
        self.unit_kind_toolbar.draw(surface, dest);
        self.draw_mode_toolbar.draw(surface, dest);
    }
}

pub struct BuildingToolSubView {
    building_kind_toolbar: TypedToolbarView<BuildingKind>,
    draw_mode_toolbar: TypedToolbarView<DrawMode>
}
impl BuildingToolSubView {
    pub fn make() -> Self {
        let default_state = BuildingToolState::default();
        let building_kind_toolbar = TypedToolbarView::make(
            0, 24,
            Rect {
                x_range: 164..164+115,
                y_range: 235..235+65
            },
            ToolbarKind::Vertical,
            default_state.building_kind
        );
        let draw_mode_toolbar = TypedToolbarView::make(
            297, 24,
            Rect {
                x_range: 164..164+115,
                y_range: 121..121+107
            },
            ToolbarKind::Vertical,
            default_state.draw_mode
        );
        Self {
            building_kind_toolbar,
            draw_mode_toolbar
        }
    }
}
impl UiView<BuildingToolState> for BuildingToolSubView {
    fn init(&mut self, model: BuildingToolState) {
        self.building_kind_toolbar.init(model.building_kind);
        self.draw_mode_toolbar.init(model.draw_mode);
    }

    fn on_button_down(&mut self) {
        self.building_kind_toolbar.on_button_down();
        self.draw_mode_toolbar.on_button_down();
    }

    fn on_button_up(&mut self) {
        self.building_kind_toolbar.on_button_up();
        self.draw_mode_toolbar.on_button_up();
    }

    fn update(&mut self, mouse_pos: (f32, f32), surface: &BlittableSurface) -> BuildingToolState {
        BuildingToolState {
            building_kind: self.building_kind_toolbar.update(mouse_pos, surface),
            draw_mode: self.draw_mode_toolbar.update(mouse_pos, surface)
        }
    }

    fn draw(&self, surface: &BlittableSurface, dest: &mut RetroBlitContext) {
        self.building_kind_toolbar.draw(surface, dest);
        self.draw_mode_toolbar.draw(surface, dest);
    }
}

pub struct ToolsAppStateView {
    tool_toolbar: TypedToolbarView<Tool>,
    terrain_tool_subview: TerrainToolSubView,
    nature_tool_subview: NatureToolSubView,
    mountain_tool_subview: MountainToolSubView,
    prop_tool_subview: PropToolSubView,
    road_tool_subview: RoadToolSubView,
    unit_tool_subview: UnitToolSubView,
    building_tool_subview: BuildingToolSubView,
}
impl ToolsAppStateView {
    pub fn make() -> Self {
        let tool_toolbar = TypedToolbarView::make(
            82, 0,
            Rect {
                x_range: 0..156,
                y_range: 0..120
            },
            ToolbarKind::Horizontal,
            Some(Tool::Terrain)
        );
        Self {
            tool_toolbar,
            terrain_tool_subview: TerrainToolSubView::make(),
            nature_tool_subview: NatureToolSubView::make(),
            mountain_tool_subview: MountainToolSubView::make(),
            prop_tool_subview: PropToolSubView::make(),
            road_tool_subview: RoadToolSubView::make(),
            unit_tool_subview: UnitToolSubView::make(),
            building_tool_subview: BuildingToolSubView::make()
        }
    }
}
impl ToolsAppStateView {
    pub fn init(&mut self, model: ToolsAppState) {
        self.tool_toolbar.init(model.tool);
        self.terrain_tool_subview.init(model.terrain_tool_state);
        self.nature_tool_subview.init(model.nature_tool_state);
        self.mountain_tool_subview.init(model.mountain_tool_state);
        self.prop_tool_subview.init(model.prop_tool_state);
        self.road_tool_subview.init(model.road_tool_state);
        self.unit_tool_subview.init(model.unit_tool_state);
        self.building_tool_subview.init(model.building_tool_state);
    }

    pub fn on_button_down(&mut self) {
        self.tool_toolbar.on_button_down();
        match self.tool_toolbar.get_selection() {
            Some(Tool::Terrain) => {
                self.terrain_tool_subview.on_button_down();
            },
            Some(Tool::Nature) => {
                self.nature_tool_subview.on_button_down();
            },
            Some(Tool::Mountains) => {
                self.mountain_tool_subview.on_button_down();
            },
            Some(Tool::Props) => {
                self.prop_tool_subview.on_button_down();
            },
            Some(Tool::Roads) => {
                self.road_tool_subview.on_button_down();
            },
            Some(Tool::Units) => {
                self.unit_tool_subview.on_button_down();
            },
            Some(Tool::Buildings) => {
                self.building_tool_subview.on_button_down();
            }
            _ => {}
        }
    }

    pub fn on_button_up(&mut self) {
        self.tool_toolbar.on_button_up();
        match self.tool_toolbar.get_selection() {
            Some(Tool::Terrain) => {
                self.terrain_tool_subview.on_button_up();
            },
            Some(Tool::Nature) => {
                self.nature_tool_subview.on_button_up();
            },
            Some(Tool::Mountains) => {
                self.mountain_tool_subview.on_button_up();
            },
            Some(Tool::Props) => {
                self.prop_tool_subview.on_button_up();
            },
            Some(Tool::Roads) => {
                self.road_tool_subview.on_button_up();
            },
            Some(Tool::Units) => {
                self.unit_tool_subview.on_button_up();
            },
            Some(Tool::Buildings) => {
                self.building_tool_subview.on_button_up();
            }
            _ => {}
        }
    }

    pub fn update(&mut self, mouse_pos: (f32, f32), surface: &BlittableSurface, old_state: ToolsAppState) -> ToolsAppState {
        let tool = self.tool_toolbar.update(mouse_pos, surface);
        match self.tool_toolbar.get_selection() {
            Some(Tool::Terrain) => {
                ToolsAppState {
                    tool,
                    terrain_tool_state: self.terrain_tool_subview.update(mouse_pos, surface),
                    ..old_state
                }
            },
            Some(Tool::Nature) => {
                ToolsAppState {
                    tool,
                    nature_tool_state: self.nature_tool_subview.update(mouse_pos, surface),
                    ..old_state
                }
            },
            Some(Tool::Mountains) => {
                ToolsAppState {
                    tool,
                    mountain_tool_state: self.mountain_tool_subview.update(mouse_pos, surface),
                    ..old_state
                }
            },
            Some(Tool::Props) => {
                ToolsAppState {
                    tool,
                    prop_tool_state: self.prop_tool_subview.update(mouse_pos, surface),
                    ..old_state
                }
            },
            Some(Tool::Roads) => {
                ToolsAppState {
                    tool,
                    road_tool_state: self.road_tool_subview.update(mouse_pos, surface),
                    ..old_state
                }
            },
            Some(Tool::Units) => {
                ToolsAppState {
                    tool,
                    unit_tool_state: self.unit_tool_subview.update(mouse_pos, surface),
                    ..old_state
                }
            },
            Some(Tool::Buildings) => {
                ToolsAppState {
                    tool,
                    building_tool_state: self.building_tool_subview.update(mouse_pos, surface),
                    ..old_state
                }
            }
            _ => old_state
        }
    }

    pub fn draw(&self, surface: &BlittableSurface, dest: &mut RetroBlitContext) {
        self.tool_toolbar.draw(surface, dest);
        match self.tool_toolbar.get_selection() {
            Some(Tool::Terrain) => {
                self.terrain_tool_subview.draw(surface, dest);
            },
            Some(Tool::Nature) => {
                self.nature_tool_subview.draw(surface, dest);
            },
            Some(Tool::Mountains) => {
                self.mountain_tool_subview.draw(surface, dest);
            },
            Some(Tool::Props) => {
                self.prop_tool_subview.draw(surface, dest);
            },
            Some(Tool::Roads) => {
                self.road_tool_subview.draw(surface, dest);
            },
            Some(Tool::Units) => {
                self.unit_tool_subview.draw(surface, dest);
            },
            Some(Tool::Buildings) => {
                self.building_tool_subview.draw(surface, dest);
            }
            _ => {}
        }
    }
}