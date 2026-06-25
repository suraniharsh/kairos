//! XDG Base Directory resolution shared between `config` and `theme` loaders.

use std::path::PathBuf;

/// Resolve the XDG base config directory. Per the XDG Base Directory Spec,
/// `XDG_CONFIG_HOME` MUST be an absolute path; relative values are ignored.
/// We warn once so users debugging path resolution can see why their relative
/// override didn't take effect.
pub fn config_home() -> Option<PathBuf> {
    if let Some(v) = std::env::var_os("XDG_CONFIG_HOME")
        && !v.is_empty()
    {
        let p = PathBuf::from(&v);
        if p.is_absolute() {
            return Some(p);
        }
        eprintln!(
            "kairos: ignoring non-absolute XDG_CONFIG_HOME={:?} (per XDG spec)",
            p.display()
        );
    }
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".config"))
}
