use std::collections::HashSet;
use crate::processors::assets::adapter::collider_adapter::Collider;
use crate::processors::assets::adapter::mesh_adapter::Mesh;
use crate::processors::assets::adapter::placeholder_adapter::Placeholder;
use crate::processors::assets::adapter::skeleton_adapter::Skeleton;
use crate::processors::assets::extras_adapter::node_role_extras_adapter::{
    NodeRoleExtras, NodeRoleType,
};
use anyhow::Result;
use gltf::{Document, Node};

pub struct AssetModel {
    pub meshes: Vec<Mesh>,
    pub colliders: Vec<Collider>,
    pub placeholders: Vec<Placeholder>,
    pub skeletons: Vec<Skeleton>,
}

impl AssetModel {
    pub fn from_document(document: &Document, bin: Option<&[u8]>) -> Result<Self> {
        let bones = collect_bones(document);

        let mut meshes = Vec::new();
        let mut colliders = Vec::new();
        let mut placeholders = Vec::new();
        let mut skeletons = Vec::new();

        for node in document.nodes() {
            if bones.contains(&node.index()) {
                continue;
            }

            let role = NodeRoleExtras::adapt(&node)?;

            match role.amberlume_type {
                NodeRoleType::Mesh => meshes.push(Mesh::adapt(&node, bin)?),
                NodeRoleType::Collider => colliders.push(Collider::adapt(&node, bin)?),
                NodeRoleType::Placeholder => placeholders.push(Placeholder::adapt(&node)?),
                NodeRoleType::Skeleton => skeletons.push(Skeleton::adapt(&node, document, bin)?),
            }
        }

        Ok(Self {
            meshes,
            colliders,
            placeholders,
            skeletons,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.meshes.is_empty()
            && self.colliders.is_empty()
            && self.placeholders.is_empty()
            && self.skeletons.is_empty()
    }
}

fn collect_bones(document: &Document) -> HashSet<usize> {
    let mut bones = HashSet::new();

    for node in document.nodes() {
        let Ok(role) = NodeRoleExtras::adapt(&node) else {
            continue;
        };

        if role.amberlume_type != NodeRoleType::Skeleton {
            continue;
        }

        for child in node.children() {
            collect_descendants(&child, &mut bones);
        }
    }

    bones
}

fn collect_descendants(node: &Node, bones: &mut HashSet<usize>) {
    bones.insert(node.index());

    for child in node.children() {
        collect_descendants(&child, bones);
    }
}
