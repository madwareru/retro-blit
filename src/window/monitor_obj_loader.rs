use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MonitorObjLoadingError {
    #[error("Float parse failed")]
    FailedToParseFloat(#[from] std::num::ParseFloatError),
    #[error("Int parse failed")]
    FailedToParseInt(#[from] std::num::ParseIntError),
    #[error("Found bad entry")]
    FoundBadEntry,
    #[error("Failed to find object name")]
    FailedToFindObjectName,
    #[error("Failed to find vertex id")]
    VertexIdNotFound,
    #[error("Failed to find UV id")]
    UVIdNotFound,
    #[error("Expected vertex component but found nothing")]
    VertexComponentExpected,
    #[error("Expected uv component but found nothing")]
    UVComponentExpected,
    #[error("Expected face component but found nothing")]
    FaceComponentExpected
}

const FILE_CONTENT:&str = include_str!("monitor_flat.obj");

enum ObjEntry {
    // we have several entires in obj file:
    // 1. the comments which start by #
    // 2. object markers which start by o
    // 3. vertices (start by v)
    // 4. uv coords (start by vt)
    // 5. faces (start by f)
    // 6. shading marker (starts with s). We ignore it for our purposes, so we will just read it as a comment
    CommentLine,
    ObjectMarker{ object_name: String},
    Vertex([f32; 3]),
    UV([f32; 2]),
    Face([(usize, usize); 3])
}

fn read_entries(file_content: &str) -> Result<Vec<ObjEntry>, MonitorObjLoadingError> {
    let mut result = Vec::new();
    let mut lines = file_content.lines();
    while let Some(line) = lines.next() {
        let mut splitted = line.split_whitespace(); // just get an iterator on slices separated by whitespace
        match splitted.next() {
            None => return Err(MonitorObjLoadingError::FoundBadEntry),
            Some(leading) => {
                if leading.starts_with("#") {
                    result.push(ObjEntry::CommentLine);
                    continue;
                }
                match leading {
                    "s" => {
                        result.push(ObjEntry::CommentLine);
                    },
                    "o" => {
                        let object_name = splitted
                            .next()
                            .ok_or(MonitorObjLoadingError::FailedToFindObjectName)?
                            .to_string();
                        result.push(ObjEntry::ObjectMarker {object_name});
                    },
                    "v" => {
                        let mut vertex = [0.0f32; 3];
                        for i in 0..3 {
                            let v_comp = splitted
                                .next()
                                .ok_or(MonitorObjLoadingError::VertexComponentExpected)?;
                            let v_comp = f32::from_str(v_comp)?;
                            vertex[i] = v_comp;
                        }
                        result.push(ObjEntry::Vertex(vertex));
                    },
                    "vt" => {
                        let mut uvs = [0.0f32; 2];
                        for i in 0..2 {
                            let uv_comp = splitted
                                .next()
                                .ok_or(MonitorObjLoadingError::UVComponentExpected)?;
                            let uv_comp = f32::from_str(uv_comp)?;
                            uvs[i] = uv_comp;
                        }
                        result.push(ObjEntry::UV(uvs));
                    },
                    "f" => {
                        let mut face_comps = [(0, 0); 3];
                        for i in 0..3 {
                            let next_couple = splitted
                                .next()
                                .ok_or(MonitorObjLoadingError::FaceComponentExpected)?;
                            face_comps[i] = parse_face_id(next_couple)?;
                        }
                        result.push(ObjEntry::Face(face_comps))
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(result)
}

fn parse_face_id(face_id_str: &str) -> Result<(usize, usize), MonitorObjLoadingError> {
    let mut face_comps = face_id_str.split("/");

    let vertex_id = face_comps.next().ok_or(MonitorObjLoadingError::VertexIdNotFound)?;
    let vertex_id = usize::from_str(vertex_id)?;

    let uv_id = face_comps.next().ok_or(MonitorObjLoadingError::UVIdNotFound)?;
    let uv_id = usize::from_str(uv_id)?;

    Ok((vertex_id, uv_id))
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32
}

impl From<[f32; 3]> for Vec4 {
    fn from(source: [f32; 3]) -> Self {
        Self {
            x:source[0],
            y:source[1],
            z:source[2],
            w: 1.0
        }
    }
}
unsafe impl bytemuck::Zeroable for Vec4{}
unsafe impl bytemuck::Pod for Vec4{}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32
}
impl From<[f32; 2]> for Vec2 {
    fn from(source: [f32; 2]) -> Self {
        Self {
            x:source[0],
            y:source[1],
        }
    }
}
unsafe impl bytemuck::Zeroable for Vec2{}
unsafe impl bytemuck::Pod for Vec2{}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    pub position: Vec4,
    pub uv: Vec2
}
unsafe impl bytemuck::Zeroable for Vertex{}
unsafe impl bytemuck::Pod for Vertex{}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>
}

impl Mesh {
    pub fn load_meshes() -> Result<HashMap<String, Self>, MonitorObjLoadingError>{
        Self::read_from_obj(FILE_CONTENT)
    }

    fn read_from_obj(file_content: &str) -> Result<HashMap<String, Self>, MonitorObjLoadingError> {
        let mut current_name = String::new();
        let mut positions = Vec::new();
        let mut uvs = Vec::new();
        let mut faces = Vec::new();
        let mut result = HashMap::new();
        let entries = read_entries(file_content)?;
        for entry in entries {
            match entry {
                ObjEntry::ObjectMarker { object_name } => {
                    if faces.len() > 0 {
                        let mesh = Self::make_mesh(&positions, &uvs, &faces);
                        result.insert(current_name, mesh);
                        faces.clear();
                    }
                    current_name = object_name;
                }
                ObjEntry::Vertex(vert_entry) => { positions.push(vert_entry) }
                ObjEntry::UV(uv_entry) => { uvs.push(uv_entry) }
                ObjEntry::Face(face_entry) => { faces.push(face_entry) }
                ObjEntry::CommentLine => {}
            }
        }
        // we need to add last mesh too
        let mesh = Self::make_mesh(&positions, &uvs, &faces);
        result.insert(current_name, mesh);
        Ok(result)
    }

    pub fn make_empty() -> Mesh {
        Self::make_mesh(&[], &[], &[])
    }

    pub fn make_square() -> Mesh {
        Self::make_mesh(
            &[
                [-1.0, -1.0, 0.0],
                [ 1.0,  1.0, 0.0],
                [-1.0,  1.0, 0.0],
                [ 1.0, -1.0, 0.0],
            ],
            &[
                [0.0, 0.0],
                [1.0, 1.0],
                [0.0, 1.0],
                [1.0, 0.0],
            ],
            &[
                [(1, 1), (2, 2), (3, 3)],
                [(1, 1), (4, 4), (2, 2)],
            ]
        )
    }

    pub fn make_4x3() -> Mesh {
        const ASPECT_X: f32 = 4.0 / 3.0;
        Self::make_mesh(
            &[
                [-ASPECT_X, -1.0, 0.0],
                [ ASPECT_X,  1.0, 0.0],
                [-ASPECT_X,  1.0, 0.0],
                [ ASPECT_X, -1.0, 0.0],
            ],
            &[
                [0.0, 0.0],
                [1.0, 1.0],
                [0.0, 1.0],
                [1.0, 0.0],
            ],
            &[
                [(1, 1), (2, 2), (3, 3)],
                [(1, 1), (4, 4), (2, 2)],
            ]
        )
    }

    pub fn make_16x10() -> Mesh {
        const ASPECT_X: f32 = 16.0 / 10.0;
        Self::make_mesh(
            &[
                [-ASPECT_X, -1.0, 0.0],
                [ ASPECT_X,  1.0, 0.0],
                [-ASPECT_X,  1.0, 0.0],
                [ ASPECT_X, -1.0, 0.0],
            ],
            &[
                [0.0, 0.0],
                [1.0, 1.0],
                [0.0, 1.0],
                [1.0, 0.0],
            ],
            &[
                [(1, 1), (2, 2), (3, 3)],
                [(1, 1), (4, 4), (2, 2)],
            ]
        )
    }

    fn make_mesh(positions: &[[f32; 3]], uvs: &[[f32; 2]], faces: &[[(usize, usize); 3]]) -> Mesh {
        let vertices = faces
            .iter()
            .flat_map(|it: &[(usize, usize); 3]| {
                it.iter().map(|&(v_id, uv_id)| {
                    Vertex {
                        position: positions[v_id - 1].into(),
                        uv: uvs[uv_id - 1].into()
                    }
                })
            })
            .collect::<Vec<Vertex>>();

        let indices = vertices // just a 0, 1, 2, 3, .. face_amount*3
            .iter()
            .enumerate()
            .map(|(ix, _)| ix as u16)
            .collect();
        Mesh { vertices, indices }
    }
}