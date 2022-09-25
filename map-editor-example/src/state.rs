#[derive(Copy, Clone)]
pub enum Tool {
    Terrain,
    Nature,
    Mountains,
    Props,
    Roads,
    Units,
    Buildings
}
impl Into<u8> for Tool {
    fn into(self) -> u8 {
        match self {
            Tool::Terrain => 1,
            Tool::Nature => 2,
            Tool::Mountains => 3,
            Tool::Props => 4,
            Tool::Roads => 5,
            Tool::Units => 6,
            Tool::Buildings => 7
        }
    }
}
impl TryFrom<u8> for Tool {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value
        {
            1 => Ok(Tool::Terrain),
            2 => Ok(Tool::Nature),
            3 => Ok(Tool::Mountains),
            4 => Ok(Tool::Props),
            5 => Ok(Tool::Roads),
            6 => Ok(Tool::Units),
            7 => Ok(Tool::Buildings),
            _ => Err(())
        }
    }
}

#[derive(Copy, Clone)]
pub enum TerrainTile {
    Rocks,
    Dirt,
    Grass,
    Sand,
    Water
}
impl Into<u8> for TerrainTile {
    fn into(self) -> u8 {
        match self {
            TerrainTile::Rocks => 8,
            TerrainTile::Dirt => 9,
            TerrainTile::Grass => 10,
            TerrainTile::Sand => 11,
            TerrainTile::Water => 12,
        }
    }
}
impl TryFrom<u8> for TerrainTile {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value
        {
            8 => Ok(TerrainTile::Rocks),
            9 => Ok(TerrainTile::Dirt),
            10 => Ok(TerrainTile::Grass),
            11 => Ok(TerrainTile::Sand),
            12 => Ok(TerrainTile::Water),
            _ => Err(())
        }
    }
}

#[derive(Copy, Clone)]
pub enum NatureKind {
    Forest,
    Bush,
    Cactus
}
impl Into<u8> for NatureKind {
    fn into(self) -> u8 {
        match self {
            NatureKind::Forest => 13,
            NatureKind::Cactus => 14,
            NatureKind::Bush => 15,
        }
    }
}
impl TryFrom<u8> for NatureKind {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value
        {
            13 => Ok(NatureKind::Forest),
            14 => Ok(NatureKind::Cactus),
            15 => Ok(NatureKind::Bush),
            _ => Err(())
        }
    }
}

#[derive(Copy, Clone)]
pub enum PropKind {
    Prop0,
    Prop1,
    Prop2,
    Prop3,
    Prop4,
    Prop5
}
impl Into<u8> for PropKind {
    fn into(self) -> u8 {
        match self {
            PropKind::Prop0 => 28,
            PropKind::Prop1 => 29,
            PropKind::Prop2 => 30,
            PropKind::Prop3 => 31,
            PropKind::Prop4 => 32,
            PropKind::Prop5 => 33,
        }
    }
}
impl TryFrom<u8> for PropKind {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value
        {
            28 => Ok(PropKind::Prop0),
            29 => Ok(PropKind::Prop1),
            30 => Ok(PropKind::Prop2),
            31 => Ok(PropKind::Prop3),
            32 => Ok(PropKind::Prop4),
            33 => Ok(PropKind::Prop5),
            _ => Err(())
        }
    }
}

#[derive(Copy, Clone)]
pub enum DrawMode {
    DrawRed,
    DrawPurple,
    DrawBlue,
    Erase
}
impl Into<u8> for DrawMode {
    fn into(self) -> u8 {
        match self {
            DrawMode::DrawRed => 16,
            DrawMode::DrawPurple => 17,
            DrawMode::DrawBlue => 18,
            DrawMode::Erase => 19,
        }
    }
}
impl TryFrom<u8> for DrawMode {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value
        {
            16 => Ok(DrawMode::DrawRed),
            17 => Ok(DrawMode::DrawPurple),
            18 => Ok(DrawMode::DrawBlue),
            19 => Ok(DrawMode::Erase),
            _ => Err(())
        }
    }
}

#[derive(Copy, Clone)]
pub enum UnitKind {
    SwordMan,
    PikeMan,
    Archer,
    CrossBowMan,
    BattleMage,
    SupportMage,
    WhiteMage,
    Knight
}
impl Into<u8> for UnitKind {
    fn into(self) -> u8 {
        match self {
            UnitKind::SwordMan => 34,
            UnitKind::PikeMan => 35,
            UnitKind::Archer => 36,
            UnitKind::CrossBowMan => 37,
            UnitKind::WhiteMage => 38,
            UnitKind::SupportMage => 39,
            UnitKind::BattleMage => 40,
            UnitKind::Knight => 41,
        }
    }
}
impl TryFrom<u8> for UnitKind {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value
        {
            34 => Ok(UnitKind::SwordMan),
            35 => Ok(UnitKind::PikeMan),
            36 => Ok(UnitKind::Archer),
            37 => Ok(UnitKind::CrossBowMan),
            38 => Ok(UnitKind::WhiteMage),
            39 => Ok(UnitKind::SupportMage),
            40 => Ok(UnitKind::BattleMage),
            41 => Ok(UnitKind::Knight),
            _ => Err(())
        }
    }
}

#[derive(Copy, Clone)]
pub enum BuildingKind {
    Keep,
    Village,
    Barracks
}
impl Into<u8> for BuildingKind {
    fn into(self) -> u8 {
        match self {
            BuildingKind::Village => 25,
            BuildingKind::Barracks => 26,
            BuildingKind::Keep => 27,
        }
    }
}
impl TryFrom<u8> for BuildingKind {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value
        {
            25 => Ok(BuildingKind::Village),
            26 => Ok(BuildingKind::Barracks),
            27 => Ok(BuildingKind::Keep),
            _ => Err(())
        }
    }
}

#[derive(Copy, Clone)]
pub enum BrushSize {
    Pixel,
    Cross,
    Square,
    Circular
}
impl Into<u8> for BrushSize {
    fn into(self) -> u8 {
        match self {
            BrushSize::Pixel => 21,
            BrushSize::Cross => 22,
            BrushSize::Square => 23,
            BrushSize::Circular => 24,
        }
    }
}
impl TryFrom<u8> for BrushSize {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value
        {
            21 => Ok(BrushSize::Pixel),
            22 => Ok(BrushSize::Cross),
            23 => Ok(BrushSize::Square),
            24 => Ok(BrushSize::Circular),
            _ => Err(())
        }
    }
}

#[derive(Copy, Clone)]
pub struct TerrainToolState {
    pub tile: Option<TerrainTile>,
    pub brush_size: Option<BrushSize>
}
impl Default for TerrainToolState {
    fn default() -> Self {
        Self {
            tile: Some(TerrainTile::Rocks),
            brush_size: Some(BrushSize::Pixel)
        }
    }
}

#[derive(Copy, Clone)]
pub struct NatureToolState {
    pub nature_kind: Option<NatureKind>,
    pub draw_mode: Option<DrawMode>,
    pub brush_size: Option<BrushSize>
}
impl Default for NatureToolState {
    fn default() -> Self {
        Self {
            nature_kind: Some(NatureKind::Forest),
            draw_mode: Some(DrawMode::DrawBlue),
            brush_size: Some(BrushSize::Pixel)
        }
    }
}

#[derive(Copy, Clone)]
pub struct MountainToolState {
    pub draw_mode: Option<DrawMode>,
    pub brush_size: Option<BrushSize>
}
impl Default for MountainToolState {
    fn default() -> Self {
        Self {
            draw_mode: Some(DrawMode::DrawBlue),
            brush_size: Some(BrushSize::Pixel)
        }
    }
}

#[derive(Copy, Clone)]
pub struct PropToolState {
    pub prop_kind: Option<PropKind>,
    pub draw_mode: Option<DrawMode>
}
impl Default for PropToolState {
    fn default() -> Self {
        Self {
            prop_kind: Some(PropKind::Prop0),
            draw_mode: Some(DrawMode::DrawBlue)
        }
    }
}

#[derive(Copy, Clone)]
pub struct RoadToolState {
    pub draw_mode: Option<DrawMode>
}
impl Default for RoadToolState {
    fn default() -> Self {
        Self {
            draw_mode: Some(DrawMode::DrawBlue)
        }
    }
}

#[derive(Copy, Clone)]
pub struct UnitToolState {
    pub unit_kind: Option<UnitKind>,
    pub draw_mode: Option<DrawMode>
}
impl Default for UnitToolState {
    fn default() -> Self {
        Self {
            unit_kind: Some(UnitKind::SwordMan),
            draw_mode: Some(DrawMode::DrawBlue)
        }
    }
}

#[derive(Copy, Clone)]
pub struct BuildingToolState {
    pub building_kind: Option<BuildingKind>,
    pub draw_mode: Option<DrawMode>
}
impl Default for BuildingToolState {
    fn default() -> Self {
        Self {
            building_kind: Some(BuildingKind::Keep),
            draw_mode: Some(DrawMode::DrawBlue)
        }
    }
}

#[derive(Copy, Clone)]
pub struct ToolsAppState {
    pub tool: Option<Tool>,
    pub terrain_tool_state: TerrainToolState,
    pub nature_tool_state: NatureToolState,
    pub mountain_tool_state: MountainToolState,
    pub prop_tool_state: PropToolState,
    pub road_tool_state: RoadToolState,
    pub unit_tool_state: UnitToolState,
    pub building_tool_state: BuildingToolState
}
impl Default for ToolsAppState {
    fn default() -> Self {
        Self {
            tool: Some(Tool::Terrain),
            terrain_tool_state: Default::default(),
            nature_tool_state: Default::default(),
            mountain_tool_state: Default::default(),
            prop_tool_state: Default::default(),
            road_tool_state: Default::default(),
            unit_tool_state: Default::default(),
            building_tool_state: Default::default()
        }
    }
}