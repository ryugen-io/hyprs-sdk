fn main() {
    #[cfg(feature = "plugin-ffi")]
    {
        // Probe pkg-config for Hyprland development headers.
        let hyprland = pkg_config::Config::new()
            .atleast_version("0.53.0")
            .probe("hyprland")
            .expect(
                "pkg-config could not find 'hyprland'. \
                 Install hyprland development headers (e.g. `hyprpm headers install`). \
                 The plugin-ffi feature requires Hyprland headers to compile the C++ bridge.",
            );

        let bridge_dir = "src/plugin/bridge";
        let bridge_files = [
            "bridge_memory.cpp",
            "bridge_config.cpp",
            "bridge_hooks.cpp",
            "bridge_hyprctl.cpp",
            "bridge_dispatch.cpp",
            "bridge_layout.cpp",
            "bridge_decoration.cpp",
            "bridge_misc.cpp",
            "bridge_lifecycle.cpp",
            "bridge_compat.cpp",
        ];

        let mut build = cc::Build::new();
        build
            .cpp(true)
            .std("c++2b")
            .pic(true)
            .warnings(false) // Hyprland headers have warnings we can't control
            .flag("-Wno-c++11-narrowing")
            .include(bridge_dir); // for common.h

        for file in &bridge_files {
            build.file(format!("{bridge_dir}/{file}"));
        }

        // Add include paths from pkg-config.
        for path in &hyprland.include_paths {
            build.include(path);
        }

        // Also probe hyprlang for its headers (used by CConfigValue).
        if let Ok(hyprlang) = pkg_config::probe_library("hyprlang") {
            for path in &hyprlang.include_paths {
                build.include(path);
            }
        }

        // hyprutils for memory types (SP, UP, etc.)
        if let Ok(hyprutils) = pkg_config::probe_library("hyprutils") {
            for path in &hyprutils.include_paths {
                build.include(path);
            }
        }

        // hyprgraphics for CColor
        if let Ok(hyprgraphics) = pkg_config::probe_library("hyprgraphics") {
            for path in &hyprgraphics.include_paths {
                build.include(path);
            }
        }

        build.compile("hyprland_bridge");

        // Tell cargo to re-run if bridge source changes.
        println!("cargo:rerun-if-changed={bridge_dir}/common.h");
        for file in &bridge_files {
            println!("cargo:rerun-if-changed={bridge_dir}/{file}");
        }
    }
}
