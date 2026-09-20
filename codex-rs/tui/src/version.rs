use std::sync::OnceLock;

use codex_build_info::BuildInfo;

/// The current Codex CLI version as embedded at compile time.
pub const CODEX_CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The release version, or abbreviated commit for an unpackaged source build.
pub(crate) fn codex_cli_version_for_display() -> &'static str {
    static DISPLAY_VERSION: OnceLock<String> = OnceLock::new();

    DISPLAY_VERSION
        .get_or_init(|| build_info_version_for_display(&BuildInfo::get()))
        .as_str()
}

fn build_info_version_for_display(build_info: &BuildInfo) -> String {
    if build_info.is_source_build() {
        let commit = build_info.build_commit();
        if let Some(commit) = commit
            .get(..8)
            .filter(|commit| commit.bytes().all(|byte| byte.is_ascii_hexdigit()))
        {
            return commit.to_string();
        }
    }

    build_info.version().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_build_displays_abbreviated_commit() {
        let build_info = BuildInfo::from_version("0123456789abcdef0123456789abcdef01234567");

        assert_eq!(build_info_version_for_display(&build_info), "01234567");
    }

    #[test]
    fn release_build_displays_package_version() {
        let build_info = BuildInfo::from_version("1.2.3-alpha.4");

        assert_eq!(build_info_version_for_display(&build_info), "1.2.3-alpha.4");
    }

    #[test]
    fn unstamped_source_build_falls_back_to_cargo_version() {
        let build_info = BuildInfo::from_version("0.0.0");

        assert_eq!(build_info_version_for_display(&build_info), "0.0.0");
    }
}
