//! Terminal window-title rendering.
//!
//! Produces a stable `kairos <path>` title so the window/tab title is
//! consistent across terminals and operating systems, rather than each
//! terminal inventing its own. The home directory collapses to `~`, and when
//! the title would exceed a fixed character budget the leading directory
//! components are progressively shortened to a single character (vim's
//! `pathshorten()` style) — but only as much as needed to fit, and never the
//! filename itself.

use std::path::Path;

/// The fixed budget, in characters, for the whole title string. The window
/// title bar's width isn't queryable across terminals/OSes, so a constant is
/// the pragmatic choice.
pub const DEFAULT_BUDGET: usize = 64;

const PREFIX: &str = "kairos ";

/// Build the terminal title for `path`. `home`, when supplied, collapses to
/// `~`. The returned string never exceeds `budget` characters unless even a
/// fully-shortened path plus the filename is longer (the filename is never
/// abbreviated).
pub fn terminal_title(path: &Path, home: Option<&Path>, budget: usize) -> String {
    let (prefix, rest) = split_display(path, home);

    // Start with nothing shortened; the last component (filename) is never
    // shortened, so only `rest[..len-1]` are candidates.
    let mut shortened = vec![false; rest.len()];
    let collapsible = rest.len().saturating_sub(1);

    loop {
        let title = render(&prefix, &rest, &shortened);
        if title.chars().count() <= budget {
            return title;
        }
        match (0..collapsible).find(|&i| !shortened[i]) {
            Some(i) => shortened[i] = true,
            // Everything collapsible is already collapsed; this is the floor.
            None => return title,
        }
    }
}

/// Split `path` into a display prefix (`"~/"`, `"/"`, or `""`) and its
/// component names (directories followed by the filename).
fn split_display(path: &Path, home: Option<&Path>) -> (String, Vec<String>) {
    if let Some(home) = home
        && let Ok(rel) = path.strip_prefix(home)
    {
        let rest = components(rel);
        if !rest.is_empty() {
            return ("~/".to_string(), rest);
        }
    }
    let prefix = if path.is_absolute() { "/" } else { "" };
    (prefix.to_string(), components(path))
}

/// Collect a path's `Normal`/`CurDir`/`ParentDir` components as strings,
/// dropping the root and any Windows prefix (rendered via the title prefix).
fn components(path: &Path) -> Vec<String> {
    use std::path::Component;
    path.components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
            Component::CurDir => Some(".".to_string()),
            Component::ParentDir => Some("..".to_string()),
            Component::RootDir | Component::Prefix(_) => None,
        })
        .collect()
}

/// Render the title, abbreviating components flagged in `shortened`.
fn render(prefix: &str, rest: &[String], shortened: &[bool]) -> String {
    let body = rest
        .iter()
        .enumerate()
        .map(|(i, name)| {
            if shortened[i] {
                short_component(name)
            } else {
                name.clone()
            }
        })
        .collect::<Vec<_>>()
        .join("/");
    format!("{PREFIX}{prefix}{body}")
}

/// Abbreviate a single path component to its first character, preserving a
/// leading dot so dotfiles stay recognizable (`.config` -> `.c`).
fn short_component(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some('.') => {
            let mut s = String::from('.');
            if let Some(c) = chars.next() {
                s.push(c);
            }
            s
        }
        Some(c) => c.to_string(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn shows_full_home_relative_path_when_it_fits() {
        let title = terminal_title(
            Path::new("/Users/m/work/todo.txt"),
            Some(Path::new("/Users/m")),
            DEFAULT_BUDGET,
        );
        assert_eq!(title, "kairos ~/work/todo.txt");
    }

    #[test]
    fn collapses_only_as_many_leading_dirs_as_needed() {
        // Full title is 51 chars; budget 35 forces collapsing through
        // `webstonehq` but leaves the deepest dir intact.
        let title = terminal_title(
            Path::new("/Users/m/projects/github/webstonehq/kairos/todo.txt"),
            Some(Path::new("/Users/m")),
            35,
        );
        assert_eq!(title, "kairos ~/p/g/w/kairos/todo.txt");
    }

    #[test]
    fn floor_collapses_all_dirs_but_keeps_filename() {
        let title = terminal_title(
            Path::new("/Users/m/projects/github/webstonehq/kairos/todo.txt"),
            Some(Path::new("/Users/m")),
            10,
        );
        assert_eq!(title, "kairos ~/p/g/w/t/todo.txt");
    }

    #[test]
    fn keeps_absolute_path_when_home_is_unknown() {
        let title = terminal_title(Path::new("/Users/m/work/todo.txt"), None, DEFAULT_BUDGET);
        assert_eq!(title, "kairos /Users/m/work/todo.txt");
    }

    #[test]
    fn preserves_leading_dot_when_shortening_dotfiles() {
        // Budget 28 forces collapsing `.config` -> `.c` but nothing more.
        let title = terminal_title(
            Path::new("/Users/m/.config/nvim/notes.txt"),
            Some(Path::new("/Users/m")),
            28,
        );
        assert_eq!(title, "kairos ~/.c/nvim/notes.txt");
    }

    #[test]
    fn shows_relative_path_without_prefix() {
        let title = terminal_title(
            Path::new("notes/todo.txt"),
            Some(Path::new("/Users/m")),
            DEFAULT_BUDGET,
        );
        assert_eq!(title, "kairos notes/todo.txt");
    }
}
