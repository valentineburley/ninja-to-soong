// Copyright 2026 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct Mesa3DDesktopTurnip {
    src_path: PathBuf,
    assets_to_filter: Vec<PathBuf>,
}

impl mesa3d_desktop::Mesa3dProject for Mesa3DDesktopTurnip {
    fn get_name(&self) -> &'static str {
        "desktop/mesa3d/turnip"
    }

    fn get_subprojects_path(&self) -> String {
        path_to_string(&self.src_path.join("subprojects"))
    }

    fn asset_filter(&self, asset: &Path) -> bool {
        !self.assets_to_filter.contains(&PathBuf::from(asset))
    }

    fn create_package(
        &mut self,
        ctx: &Context,
        src_path: &Path,
        build_path: &Path,
        ndk_path: &Path,
        meson_generated: &str,
        targets_map: NinjaTargetsMap<MesonNinjaTarget>,
    ) -> Result<SoongPackage, String> {
        self.src_path = PathBuf::from(src_path);
        let targets_to_gen = NinjaTargetsToGenMap::from(&[
            target!(
                "src/freedreno/vulkan/libvulkan_freedreno.so",
                "desktop-mesa3d_turnip_libvulkan_freedreno",
                "vulkan.freedreno"
            ),
            target!(
                "src/tool/pps/pps-producer",
                "desktop-mesa3d_turnip_pps-producer",
                "pps-producer"
            ),
            target!(
                "src/tool/pps/libgpudataproducer.so",
                "desktop-mesa3d_turnip_libgpudataproducer",
                "libgpudataproducer_freedreno"
            ),
        ]);
        self.assets_to_filter = Self::extract_assets_to_filter(&targets_to_gen, &targets_map)?;
        SoongPackage::new(
            &["//visibility:public"],
            "mesa3d_desktop_turnip_licenses",
            &[
                "SPDX-license-identifier-Apache-2.0",
                "SPDX-license-identifier-MIT",
                "SPDX-license-identifier-BSL-1.0",
            ],
            &["licenses/Apache-2.0", "licenses/MIT", "licenses/BSL-1.0"],
        )
        .generate_from_map(
            targets_to_gen,
            targets_map,
            &self.src_path,
            &ndk_path,
            &build_path,
            Some(meson_generated),
            self,
            ctx,
        )
    }

    fn get_default_module(&self, package: &SoongPackage) -> Result<SoongModule, String> {
        Ok(SoongModule::new_cc_defaults(CcDefaults::Mesa3DTurnip)
            .add_props(package.get_props("desktop-mesa3d_turnip_pps-producer", vec!["cflags"])?)
            .add_defaults(CcDefaults::Mesa3DTurnipManual)?)
    }

    fn get_raw_suffix(&self, common_raw_prop: &'static str) -> String {
        format!(
            r#"
cc_defaults {{
    name: "{}",
    soc_specific: true,
    header_libs: ["libdrm_headers"],
    static_libs: ["libperfetto_client_experimental"],
    shared_libs: ["libz"],
{common_raw_prop}
}}
"#,
            CcDefaults::Mesa3DTurnipManual.str()
        )
    }

    fn extend_module(&self, target: &Path, mut module: SoongModule) -> Result<SoongModule, String> {
        module.update_prop("generated_headers", |prop| {
            let SoongProp::VecStr(mut vec) = prop else {
                return Ok(prop);
            };
            vec.push(path_to_id(
                Path::new(mesa3d_desktop::Mesa3dProject::get_name(self))
                    .join("src/util/shader_stats.h"),
            ));
            Ok(SoongProp::VecStr(vec))
        })?;

        if target.ends_with("libvulkan_freedreno.so") {
            module = module
                .add_prop("relative_install_path", SoongProp::Str(String::from("hw")))
                .add_prop("afdo", SoongProp::Bool(true))
        }

        let mut cflags = vec![
            "-Wno-array-bounds",
            "-Wno-c++11-narrowing",
            "-Wno-missing-braces",
            "-Wno-unused-parameter",
        ];
        if target.ends_with("libvulkan_lite_runtime.a") {
            cflags.push("-Wno-unreachable-code-loop-increment");
        }
        module
            .add_defaults(CcDefaults::Mesa3DTurnip)?
            .extend_prop("cflags", cflags)
    }
}
