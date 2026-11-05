//! pygame_spine —— Spine 2D 骨骼动画适配层（阶段 4 绑定范式：
//! rapier-starfish-debug 兄弟 crate 同款——上游运行时 + 生态侧薄适配，
//! pygame-rs 经 rpy/pygame_spine 绑定面消费）。
//!
//! 上游 = [chine](https://crates.io/crates/chine)（纯 Rust Spine 运行时，
//! JSON/二进制双加载 + AnimationState 状态机 + render 命令流输出）。
//! 本 crate 直通其核心面（词汇不pygame化——Spine 惯例即文档），附加
//! 一个程序化 demo 骨架（三骨链——无资产文件即可跑通绑定/测试链）。
//!
//! 渲染接入路径（v1 数据面）：`chine::render::render(&skeleton)` 产出
//! `RenderCommand` 流（positions/uvs/triangles/page/color）——绑定层已
//! 透传为 Python 数据；引擎侧带贴图三角网格 draw 面（draw_mesh）落地后
//! 即接 GPU 管线。

pub use chine::anim::{Animation, AnimationState};
pub use chine::binary;
pub use chine::data::{BlendMode, BoneData, SkeletonData};
pub use chine::load;
pub use chine::render::{self, RenderCommand};
pub use chine::skel::{Bone, Skeleton};

use std::sync::Arc;

/// 程序化 demo 骨架（三骨链 root→arm→hand；无资产依赖——绑定/测试
/// 链即装即跑）。root 原点，arm 偏移 (50, 0)，hand 相对 arm (40, 0)。
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_chain_setup_pose() {
        let mut sk = Skeleton::new(demo_skeleton());
        sk.update_world_transform();
        let arm = sk.bone(1).unwrap();
        assert!((arm.world_x() - 50.0).abs() < 1e-4);
        let hand = sk.bone(2).unwrap();
        assert!((hand.world_x() - 90.0).abs() < 1e-4);
    }

    #[test]
    fn demo_rotation_propagates() {
        let mut sk = Skeleton::new(demo_skeleton());
        sk.bone_mut(0).unwrap().rotation = 90.0;
        sk.update_world_transform();
        let arm = sk.bone(1).unwrap();
        assert!((arm.world_x() - 0.0).abs() < 1e-4);
        assert!((arm.world_y() - 50.0).abs() < 1e-4);
    }
}
