// use crate::resources::io_provider::IOProvider;
// use dashmap::DashMap;
// use std::env::current_exe;
// use std::path::{Path, PathBuf};
// use std::sync::Arc;
// use tracing::info;
// use walkdir::WalkDir;
//
// // pub struct ModelIndexEntry {
// //     pub file_path: PathBuf,
// //     pub primitives: Vec<PrimitiveIndexEntry>,
// // }
//
// // pub struct PrimitiveIndexEntry {
// //     pub material_name: Option<String>,
// // }
//
// // pub struct MaterialIndexEntry {
// //     pub file_path: PathBuf,
// // }
//
// #[derive(Clone)]
// pub struct ShaderIndexEntry {
//     pub name: String,
//     pub file_path: PathBuf,
// }
//
// pub struct ResourceIndex {
//     io_provider: Arc<dyn IOProvider>,
//
//     pub shaders: DashMap<String, ShaderIndexEntry>,
//     // pub models: DashMap<String, ModelIndexEntry>,
//     // pub materials: DashMap<String, MaterialIndexEntry>
// }
//
// impl ResourceIndex {
//     pub fn new(io_provider: Arc<dyn IOProvider>) -> Self {
//         let shaders_path = current_exe()
//             .unwrap()
//             .parent()
//             .unwrap()
//             .join("assets/shaders");
//
//         let shaders = DashMap::new();
//         for entry in WalkDir::new(shaders_path)
//             .into_iter()
//             .filter_map(|e| e.ok())
//         {
//             let path = entry.path();
//
//             if path.extension().and_then(|ext| ext.to_str()) == Some("spv") {
//                 info!("Indexing shaders {}", path.display());
//                 let shader_index_entry = Self::index_shader_file(&path);
//
//                 let name = path.file_stem().unwrap().to_str().unwrap().to_owned();
//                 shaders.insert(name, shader_index_entry);
//             }
//         }
//
//         // for entry in WalkDir::new(&self.models_path).into_iter().filter_map(|e| e.ok()) {
//         //     let path = entry.path();
//         //
//         //     if path.extension().and_then(|ext| ext.to_str()) == Some("glb") {
//         //         self.index_model_file(&path);
//         //     }
//         // }
//
//         Self {
//             io_provider,
//
//             shaders,
//             // models: DashMap::new(),
//             // materials: DashMap::new(),
//         }
//     }
//
//     pub fn get_shader_entry(&self, name: &str) -> ShaderIndexEntry {
//         if let Some(shader_index_entry) = self.shaders.get(name) {
//             return shader_index_entry.value().clone();
//         }
//
//         panic!("Shader resources index with name '{}' not found", name);
//     }
//
//     fn index_shader_file(path: &Path) -> ShaderIndexEntry {
//         let shader_name = path.file_name().unwrap().to_str().unwrap().to_owned();
//
//         ShaderIndexEntry {
//             name: shader_name.clone(),
//             file_path: path.to_path_buf(),
//         }
//     }
//
//     // #[instrument(skip(self))]
//     // fn index_model_file(&self, path: &Path) {
//     //     let document = Gltf::open(path).unwrap();
//     //
//     //     for mesh in document.meshes() {
//     //         self.index_model_mesh(&mesh, &path);
//     //     }
//     //
//     //     for material in document.materials() {
//     //         self.index_material(&material, &path);
//     //     }
//     // }
//
//     // #[instrument(skip(self, mesh))]
//     // fn index_model_mesh(&self, mesh: &Mesh, path: &Path) {
//     //     let model_name = mesh.name().unwrap_or_else(|| {
//     //         panic!("Trying to import model from '{}' without name", path.display());
//     //     }).to_string();
//     //
//     //     let mut primitives = Vec::with_capacity(mesh.primitives().len());
//     //
//     //     for primitive in mesh.primitives() {
//     //         let primitive_index_entry = PrimitiveIndexEntry {
//     //             material_name: primitive.material().name().map(String::from),
//     //         };
//     //
//     //         primitives.push(primitive_index_entry);
//     //     }
//     //
//     //     let model_resource = ModelIndexEntry {
//     //         file_path: path.to_path_buf(),
//     //         primitives,
//     //     };
//     //
//     //     self.models.insert(model_name, model_resource);
//     // }
//
//     // #[instrument(skip(self, material))]
//     // fn index_material(&self, material: &Material, path: &Path) {
//     //     let material_name = material.name().unwrap_or_else(|| {
//     //         panic!("Trying to import material from '{}' without name", path.display());
//     //     }).to_string();
//     //
//     //     let material_resource = MaterialIndexEntry {
//     //         file_path: path.to_path_buf(),
//     //     };
//     //
//     //     self.materials.insert(material_name, material_resource);
//     // }
// }
