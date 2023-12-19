use std::path::{Path, PathBuf};

use bevy::{
    asset::{io::Reader, AssetLoader, AssetPath, LoadContext},
    ecs::system::SystemState,
    prelude::*,
    render::RenderApp,
    utils::{BoxedFuture, HashMap},
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "rust-gpu-builder")]
use spirv_builder::{MetadataPrintout, ShaderPanicStrategy, SpirvBuilder};

#[cfg(feature = "rust-gpu-builder")]
use std::{env, fs};

pub struct RustGpuPlugin;

impl Plugin for RustGpuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RustGpuRegistry>()
            .init_asset::<RustGpuShader>()
            .register_asset_loader(RustGpuLoader);
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.init_resource::<RustGpuRegistry>();
    }
}

/// Holds RustGpu shader handles so the file watcher will watch for updates and cause a new spv file to be generated when changes are made.
#[derive(Resource, Default)]
pub struct RustGpuRegistry(HashMap<PathBuf, Handle<RustGpuShader>>);

impl RustGpuRegistry {
    pub fn load<'a>(
        &mut self,
        path: impl Into<AssetPath<'a>> + std::marker::Copy,
        out_dir: impl Into<AssetPath<'a>> + std::marker::Copy,
        asset_server: &AssetServer,
    ) -> Handle<Shader> {
        let p: PathBuf = path.into().into();
        let out_dir: PathBuf = out_dir.into().into();
        let shader = asset_server.load(out_dir.clone());
        // TODO skip this if not using "file_watcher" or "asset_processor" features.
        #[cfg(not(target_arch = "wasm32"))]
        {
            let h = asset_server.load_with_settings(p.clone(), move |s: &mut RustGpuSettings| {
                s.out_dir = out_dir.clone();
            });
            self.0.insert(p.clone(), h);
        }
        shader
    }

    pub fn load_from_world<'a>(
        path: impl Into<AssetPath<'a>> + std::marker::Copy,
        out_dir: impl Into<AssetPath<'a>> + std::marker::Copy,
        world: &mut World,
    ) -> Handle<Shader> {
        let mut system_state: SystemState<(Res<AssetServer>, ResMut<RustGpuRegistry>)> =
            SystemState::new(world);
        let (asset_server, mut rustgpu) = system_state.get_mut(world);
        rustgpu.load(path, out_dir, &asset_server)
    }
}

#[derive(Asset, TypePath, Debug)]
pub struct RustGpuShader(PathBuf);

#[derive(Default)]
struct RustGpuLoader;

#[derive(Default, Serialize, Deserialize)]
struct RustGpuSettings {
    profile: String,
    out_dir: PathBuf,
}

impl AssetLoader for RustGpuLoader {
    type Asset = RustGpuShader;
    type Settings = RustGpuSettings;
    type Error = std::io::Error;

    #[allow(unused_variables)]
    fn load<'a>(
        &'a self,
        _reader: &'a mut Reader,
        settings: &'a RustGpuSettings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<RustGpuShader, Self::Error>> {
        let crate_dir: PathBuf = Path::new("assets").join(
            load_context
                .asset_path()
                .path()
                .parent()
                .unwrap()
                .parent()
                .unwrap(),
        );
        #[cfg(feature = "rust-gpu-builder")]
        {
            let builder = SpirvBuilder::new(crate_dir.clone(), "spirv-unknown-vulkan1.1")
                .print_metadata(MetadataPrintout::None)
                .extra_arg(format!("OUT_DIR={}", settings.out_dir.to_string_lossy()))
                .shader_panic_strategy(ShaderPanicStrategy::SilentExit);
            let out_dir = env::current_dir()
                .unwrap()
                .join("assets")
                .join(&settings.out_dir);
            let result = builder.build().unwrap();

            fs::copy(result.module.unwrap_single(), out_dir).unwrap();
        }

        Box::pin(async move { Ok(RustGpuShader(crate_dir.to_path_buf())) })
    }

    fn extensions(&self) -> &[&str] {
        &["rs"]
    }
}
