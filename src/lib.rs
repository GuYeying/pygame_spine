//! `chine`: a pure-Rust Spine 4.3 skeletal animation runtime.
//!
//! `chine` loads Spine skeleton exports (JSON / binary `.skel`) together with an
//! atlas, poses and animates a skeleton, and emits renderer-agnostic draw data,
//! leaving all GPU work to the host engine.
//!
//! It is a from-scratch reimplementation that references the official Spine 4.3
//! runtimes, and is **not** affiliated with or endorsed by Esoteric Software.
//! Using Spine skeleton data requires a valid Spine Editor license.
//!
//! # Pipeline
//!
//! 1. Load a JSON or binary `.skel` export plus an atlas into a
//!    `SkeletonData`: the immutable, shareable rig.
//! 2. Instantiate a `Skeleton` from it: a posable instance.
//!    `Skeleton::set_skin` shows a named skin.
//! 3. Each frame, `AnimationState::update` and `apply` drive the skeleton's
//!    local pose, `Skeleton::update_world_transform` computes the world pose
//!    (forward kinematics plus IK, transform, path, physics, and slider
//!    constraints), and the `RenderCommand` stream feeds the host renderer.
//!
//! The world transform of each bone is a 2x2 matrix `(a, b, c, d)` plus a world
//! position `(world_x, world_y)`, computed root-to-children, the same model the
//! official runtimes use.
#![warn(missing_docs)]
#![warn(unreachable_pub)]
#![warn(clippy::all)]
// Without a loader feature, the loader-fed timeline variants, channel tables,
// and data constructors are unused by design: the crate is then a manual
// pose/render runtime over a hand-built SkeletonData. Allow that dead code only
// in the no-loader configuration. The loader builds still enforce every warning.
#![cfg_attr(
    not(any(feature = "json", feature = "binary")),
    allow(dead_code, unused_imports)
)]

pub mod anim;
pub mod atlas;
pub mod attach;
#[cfg(feature = "binary")]
pub mod binary;
pub mod clip;
pub(crate) mod constraint;
pub mod data;
pub(crate) mod event;
pub(crate) mod link;
#[cfg(feature = "json")]
pub mod load;
pub mod render;
pub mod skel;
pub mod skin;

// ── pygamest 适配面（demo 构造器 + 惯用重导出）──────────────────────

pub use crate::anim::{Animation, AnimationState};
pub use crate::data::{BlendMode, BoneData, SkeletonData};
pub use crate::skel::{Bone, Skeleton};
pub use crate::render::RenderCommand;

use std::sync::Arc;

/// 程序化 demo 骨架（三骨链 root→arm→hand；无资产依赖——绑定/测试
/// 链即装即跑）。
pub fn demo_skeleton() -> Arc<SkeletonData> {
    Arc::new(SkeletonData {
        bones: vec![
            crate::data::BoneData {
                index: 0,
                name: "root".into(),
                ..Default::default()
            },
            crate::data::BoneData {
                index: 1,
                name: "arm".into(),
                parent: Some(0),
                position: glam::Vec2::new(50.0, 0.0),
                ..Default::default()
            },
            crate::data::BoneData {
                index: 2,
                name: "hand".into(),
                parent: Some(1),
                position: glam::Vec2::new(40.0, 0.0),
                ..Default::default()
            },
        ],
        ..Default::default()
    })
}

/// 带贴图 region 的 demo 骨架（三骨链 + 两 slot 两方块：body 96×96 挂
/// root、hand 48×48 挂 hand 骨——page0 纹理由调用方供给，uv = region
/// 全图。零资产渲染 demo 的数据源——与官方 render 测试同构的程序化
/// 构造路径）。
pub fn demo_textured_skeleton() -> Arc<SkeletonData> {
    use crate::attach::{Attachment, RegionAttachment};
    use crate::atlas::AtlasRegion;
    use crate::skin::Skin;
    use crate::data::SlotData;

    let region = AtlasRegion {
        name: "demo".into(),
        page: 0,
        x: 0,
        y: 0,
        width: 64,
        height: 64,
        degrees: 0,
        offset_x: 0.0,
        offset_y: 0.0,
        original_width: 64,
        original_height: 64,
        index: -1,
    };

    let mut body = RegionAttachment::new("body", "demo");
    body.width = 96.0;
    body.height = 96.0;
    body.color = crate::data::Color { r: 0.35, g: 0.75, b: 1.0, a: 1.0 };
    body.update(&region, 64, 64);
    body.page = 0;

    let mut hand = RegionAttachment::new("hand", "demo");
    hand.width = 48.0;
    hand.height = 48.0;
    hand.color = crate::data::Color { r: 1.0, g: 0.65, b: 0.3, a: 1.0 };
    hand.update(&region, 64, 64);
    hand.page = 0;

    let mut skin = Skin::new("default");
    skin.set(0, "body", Attachment::Region(body));
    skin.set(1, "hand", Attachment::Region(hand));

    Arc::new(SkeletonData {
        bones: vec![
            crate::data::BoneData {
                index: 0,
                name: "root".into(),
                ..Default::default()
            },
            crate::data::BoneData {
                index: 1,
                name: "arm".into(),
                parent: Some(0),
                position: glam::Vec2::new(70.0, 0.0),
                ..Default::default()
            },
            crate::data::BoneData {
                index: 2,
                name: "hand".into(),
                parent: Some(1),
                position: glam::Vec2::new(60.0, 0.0),
                ..Default::default()
            },
        ],
        slots: vec![
            crate::data::SlotData {
                index: 0,
                name: "body_slot".into(),
                bone: 0,
                color: crate::data::Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
                dark_color: None,
                blend: crate::data::BlendMode::Normal,
                attachment: Some("body".into()),
            },
            crate::data::SlotData {
                index: 1,
                name: "hand_slot".into(),
                bone: 2,
                color: crate::data::Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
                dark_color: None,
                blend: crate::data::BlendMode::Normal,
                attachment: Some("hand".into()),
            },
        ],
        skins: vec![skin],
        ..Default::default()
    })
}

/// JSON 骨架 + atlas 元数据一体加载并绑定（page 索引/uv 重映射就地
/// 生效）——返回（绑定完成的骨架数据, 图集页数）。页面纹理由调用方
/// 经 pygame.image.load 后传入 gfxdraw.draw_spine 的 pages 表。
pub fn load_with_atlas(
    json_text: &str,
    atlas_text: &str,
) -> Result<(Arc<SkeletonData>, usize), crate::load::LoadError> {
    let mut data = crate::load::from_json(json_text)?;
    let atlas = crate::atlas::Atlas::parse(atlas_text);
    let pages = atlas.pages.len();
    crate::render::bind_atlas(&mut data, &atlas);
    Ok((Arc::new(data), pages))
}
