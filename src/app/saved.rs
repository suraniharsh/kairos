//! Named saved searches: upsert + persistence. The `ff` recall picker lives
//! in `picker.rs` alongside its project/context siblings.
use super::App;
use super::types::SavedFilter;
use crate::config::Config;

impl App {
    /// Read-only view of the user's saved searches.
    pub fn saved_filters(&self) -> &[SavedFilter] {
        &self.saved_filters
    }

    /// Validate `name` and upsert `{name, query: <current search>}` into the
    /// in-memory list. Pure (no I/O) so it can be unit-tested without
    /// touching the user's real config. Re-saving an existing name replaces
    /// its query in place. Returns the trimmed name on success, or a
    /// user-facing error message.
    pub(crate) fn upsert_saved_filter(&mut self, name: &str) -> Result<String, String> {
        let name = name.trim();
        if name.is_empty() {
            return Err("filter name required".into());
        }
        // `=` would break the `filter.<name> = <query>` key/value split on
        // reload, silently corrupting the name.
        if name.contains('=') {
            return Err("filter name can't contain '='".into());
        }
        // Trim so the stored value equals what survives the config
        // round-trip (`parse` does `value.trim()`); a whitespace-only
        // search trims to empty and is rejected like a blank one.
        let query = self.filter.search.trim().to_string();
        if query.is_empty() {
            return Err("no active search to save".into());
        }
        match self.saved_filters.iter_mut().find(|f| f.name == name) {
            Some(existing) => existing.query = query,
            None => self.saved_filters.push(SavedFilter {
                name: name.to_string(),
                query,
            }),
        }
        Ok(name.to_string())
    }

    /// Name the current `/`-search and persist it. Flashes the outcome.
    ///
    /// Persistence is load-modify-save: scalar prefs/share keys written to
    /// the file by another process are preserved because `Config::load`
    /// re-reads them. The saved-filter block is *merged* — `merge_saved`
    /// overlays our in-memory list onto whatever `filter.*` lines are on
    /// disk, so a filter added to the config externally since startup also
    /// survives instead of being clobbered by a wholesale overwrite. (An
    /// external *deletion* is not honored: there is no in-app delete, so
    /// the startup snapshot still carries it.)
    pub fn save_current_filter_as(&mut self, name: &str) {
        match self.upsert_saved_filter(name) {
            Ok(saved) => {
                let mut cfg = Config::load();
                cfg.filters = merge_saved(&cfg.filters, &self.saved_filters);
                if let Err(e) = cfg.save() {
                    self.flash(format!("save failed: {e}"));
                } else {
                    self.flash(format!("saved filter: {saved}"));
                }
            }
            Err(msg) => self.flash(msg),
        }
    }
}

/// Overlay the in-memory saved filters onto what's currently on disk.
/// Disk order and disk-only entries are preserved; an in-memory filter
/// replaces a disk entry of the same name in place, and a name not on
/// disk is appended. Keeps `filter.*` lines added to the config file
/// externally instead of clobbering the whole block.
fn merge_saved(disk: &[(String, String)], mem: &[SavedFilter]) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = disk.to_vec();
    for f in mem {
        match out.iter_mut().find(|(n, _)| *n == f.name) {
            Some((_, q)) => *q = f.query.clone(),
            None => out.push((f.name.clone(), f.query.clone())),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use crate::app::test_support::build_app;

    #[test]
    fn upsert_adds_then_replaces_in_place() {
        let mut app = build_app(crate::sample::TODO_RAW);
        app.set_search("report".into());
        assert_eq!(app.upsert_saved_filter("weekly").as_deref(), Ok("weekly"));
        assert_eq!(app.saved_filters().len(), 1);
        assert_eq!(app.saved_filters()[0].query, "report");

        // Same name, different active search → replace, not duplicate.
        app.set_search("retro".into());
        assert_eq!(app.upsert_saved_filter("weekly").as_deref(), Ok("weekly"));
        assert_eq!(app.saved_filters().len(), 1);
        assert_eq!(app.saved_filters()[0].query, "retro");
    }

    #[test]
    fn upsert_rejects_empty_name_equals_sign_and_no_search() {
        let mut app = build_app(crate::sample::TODO_RAW);

        app.set_search("report".into());
        assert!(app.upsert_saved_filter("   ").is_err());
        assert!(app.upsert_saved_filter("a=b").is_err());
        assert!(app.saved_filters().is_empty());

        app.clear_search();
        assert!(app.upsert_saved_filter("weekly").is_err());
        assert!(app.saved_filters().is_empty());
    }

    #[test]
    fn upsert_trims_name() {
        let mut app = build_app(crate::sample::TODO_RAW);
        app.set_search("report".into());
        assert_eq!(
            app.upsert_saved_filter("  weekly  ").as_deref(),
            Ok("weekly")
        );
        assert_eq!(app.saved_filters()[0].name, "weekly");
    }

    #[test]
    fn upsert_trims_query_so_it_survives_round_trip() {
        // The config round-trip does `value.trim()` on reload, so a query
        // with surrounding whitespace would come back changed. Trim at the
        // source so the in-memory value equals what persists.
        let mut app = build_app(crate::sample::TODO_RAW);
        app.set_search("  spaced  ".into());
        assert_eq!(app.upsert_saved_filter("x").as_deref(), Ok("x"));
        assert_eq!(app.saved_filters()[0].query, "spaced");
    }

    #[test]
    fn upsert_rejects_whitespace_only_query() {
        // A whitespace-only query is non-empty before trimming but reloads
        // as empty — the same silent corruption the empty-query guard exists
        // to prevent. Reject it after trimming.
        let mut app = build_app(crate::sample::TODO_RAW);
        app.set_search("   ".into());
        assert!(app.upsert_saved_filter("x").is_err());
        assert!(app.saved_filters().is_empty());
    }

    #[test]
    fn merge_saved_preserves_disk_only_and_replaces_in_place() {
        use crate::app::SavedFilter;
        let disk = vec![
            ("weekly".to_string(), "OLD".to_string()),
            ("external".to_string(), "ext".to_string()),
        ];
        let mem = vec![
            SavedFilter {
                name: "weekly".into(),
                query: "NEW".into(),
            },
            SavedFilter {
                name: "fresh".into(),
                query: "f".into(),
            },
        ];
        assert_eq!(
            super::merge_saved(&disk, &mem),
            vec![
                ("weekly".to_string(), "NEW".to_string()), // replaced in place
                ("external".to_string(), "ext".to_string()), // disk-only kept
                ("fresh".to_string(), "f".to_string()),    // new appended
            ]
        );
    }
}
