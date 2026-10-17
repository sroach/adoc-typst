use std::fs;
use std::path::Path;

use anyhow::Result;

/// Warning icon: amber triangle with exclamation mark
pub const ICON_WARNING: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="1em" height="1em" fill="none" stroke="#F59E0B" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>"##;

/// Caution icon: orange triangle with exclamation mark
pub const ICON_CAUTION: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="1em" height="1em" fill="none" stroke="#F97316" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>"##;

/// Tip icon: green lightbulb
pub const ICON_TIP: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="1em" height="1em" fill="none" stroke="#10B981" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9.663 17h4.673M12 20h.01"/><path d="M12.43 3.57a.96.96 0 0 0-1.86 0L8.68 10.26a.96.96 0 0 0 .57 1.15l2.36.8a.96.96 0 0 1 .57 1.15l-.79 2.36a.96.96 0 0 0 1.15.57l6.69-3.71a.96.96 0 0 0 0-1.67l-6.69-3.71a.96.96 0 0 1 0-1.67z"/></svg>"##;

/// Note icon: blue pencil
pub const ICON_NOTE: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="1em" height="1em" fill="none" stroke="#3B82F6" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>"##;

/// Important icon: red circle with exclamation mark
pub const ICON_IMPORTANT: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="1em" height="1em" fill="none" stroke="#EF4444" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>"##;

/// All icon definitions: filename and SVG content
const ALL_ICONS: &[(&str, &str)] = &[
    ("icon-warning.svg", ICON_WARNING),
    ("icon-caution.svg", ICON_CAUTION),
    ("icon-tip.svg", ICON_TIP),
    ("icon-note.svg", ICON_NOTE),
    ("icon-important.svg", ICON_IMPORTANT),
];

/// Write all icon SVG files to the given directory.
/// This ensures icons are available next to the generated `.typst` output file.
pub fn write_icons_to_dir(dir: &Path) -> Result<()> {
    for (filename, content) in ALL_ICONS {
        let path = dir.join(filename);
        fs::write(&path, content)?;
    }
    Ok(())
}
