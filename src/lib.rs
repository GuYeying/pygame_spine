//! pygame_spine —— Spine 2D 骨骼动画适配层（2026-11-07 重构：**chine
//! 回归 crates.io 依赖**——vendored 树退役；本 crate 只剩薄适配面）。
//!
//! 上游 = [chine](https://crates.io/crates/chine) v0.2.0（MIT，
//! Oneiriq/chine——纯 Rust Spine 4.3 运行时：JSON/binary 加载、动画
//! 状态机、render 命令生成全 pub）。任务 4 定约（spine/ozz 以
//! crates.io 依赖引入、不自行维护源码）的 spine 侧落地。
//!
//! ⚠️ vendored 时代的三处 delta 去向：
//! 1. attach/clip/skin 提公 → **不再需要**：上游 render 模块 pub
//!    （render/render_into/render_with/RenderScratch 全可达——绑定侧
//!    RenderScratch 经 `pygame_spine::render::RenderScratch`）；
//! 2. default-skin find_skin 盲区 → 绑定层校验补 `default_skin.name`
//!    比对（字段 pub 可达；`Skeleton::set_skin` 上游语义本就正确——
//!    未命中 skins 表 = None 槽 = default 皮肤激活）；
//! 3. demo 构造器 → 本文件（[`demo_skeleton`] pub 字段构造；
//!    [`demo_textured_skeleton`] 改走内联 JSON + [`load_with_atlas`]
//!    ——公开 API 等价路径，region/uv 经 atlas 绑定全图）。
//!
//! 渲染接入 = `chine::render::render` 命令流（positions/uvs/triangles/
//! page 透传，引擎侧带贴图 mesh 绘制；RenderScratch 经会话复用）。

pub use chine::{
    anim::{Animation, AnimationState},
    binary,
    data::{BlendMode, BoneData, SkeletonData},
    load,
    render::{self, RenderCommand, RenderScratch},
    skel::{Bone, Skeleton},
    atlas::Atlas,
};

use std::sync::Arc;

/// 程序化 demo 骨架（三骨链 root→arm→hand；无资产依赖——绑定/测试
/// 链即装即跑）。
pub fn demo_skeleton() -> Arc<SkeletonData> {
    Arc::new(SkeletonData {
        bones: vec![
            BoneData {
                index: 0,
                name: "root".into(),
                ..Default::default()
            },
            BoneData {
                index: 1,
                name: "arm".into(),
                parent: Some(0),
                position: glam::Vec2::new(50.0, 0.0),
                ..Default::default()
            },
            BoneData {
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

/// demo 骨架的内联 JSON（Spine 4.2 对象 skins 形态——chine from_json
/// 直吃；region/uv 经 [`DEMO_ATLAS`] + [`load_with_atlas`] 绑定全图）。
pub const DEMO_JSON: &str = r#"{
    "skeleton": {"hash": "demo", "spine": "4.2.0", "x": -50, "y": -50, "width": 100, "height": 100},
    "bones": [
        {"name": "root"},
        {"name": "arm", "parent": "root", "x": 70, "y": 0},
        {"name": "hand", "parent": "arm", "x": 60, "y": 0}
    ],
    "slots": [
        {"name": "body_slot", "bone": "root", "attachment": "body"},
        {"name": "hand_slot", "bone": "hand", "attachment": "hand"}
    ],
    "skins": [
        {
            "name": "default",
            "attachments": {
                "body_slot": {
                    "body": {"type": "region", "path": "body", "x": 48, "y": 0, "width": 96, "height": 96}
                },
                "hand_slot": {
                    "hand": {"type": "region", "path": "hand", "x": 24, "y": 0, "width": 48, "height": 48}
                }
            }
        }
    ]
}"#;

/// demo 内联 atlas（body/hand 两 region 各占全页 64×64——uv 全图，
/// 调用方供 page0 纹理即可）。
pub const DEMO_ATLAS: &str = "demo\nsize: 64,64\nformat: RGBA8888\nfilter: Linear,Linear\nrepeat: none\nbody\n  bounds: 0, 0, 64, 64\nhand\n  bounds: 0, 0, 64, 64\n";

/// 带贴图 region 的 demo 骨架（三骨链 + body/hand 两 slot 两方块：
/// page0 纹理由调用方供给，uv = region 全图。零资产渲染 demo 的数据
/// 源——内联 JSON + atlas 经公开 API 构造，与 vendored 时代等价）。
pub fn demo_textured_skeleton() -> Arc<SkeletonData> {
    let (data, _) = load_with_atlas(DEMO_JSON, DEMO_ATLAS).expect("demo 数据内联构造");
    data
}

/// JSON 骨架 + atlas 元数据一体加载并绑定（page 索引/uv 重映射就地
/// 生效）——返回（绑定完成的骨架数据, 图集页数）。页面纹理由调用方
/// 经 pygame.image.load 后传入 gfxdraw.draw_spine 的 pages 表。
pub fn load_with_atlas(
    json_text: &str,
    atlas_text: &str,
) -> Result<(Arc<SkeletonData>, usize), chine::load::LoadError> {
    let mut data = chine::load::from_json(json_text)?;
    let atlas = Atlas::parse(atlas_text);
    let pages = atlas.pages.len();
    render::bind_atlas(&mut data, &atlas);
    Ok((Arc::new(data), pages))
}
