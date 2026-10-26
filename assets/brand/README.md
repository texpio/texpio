# Texpio brand assets

The final brand set selected on 26 October 2026 is stored directly in this directory. It replaces the earlier concepts and versioned folders.

| File | Use |
| --- | --- |
| [icon.svg](icon.svg) | Transparent green gradient t/expansion-arrow symbol. |
| [icon-with-bg.svg](icon-with-bg.svg) | Green symbol on a navy rounded-square field with a green border; source for application, installer, tray and browser icons. |
| [logo-light-theme.svg](logo-light-theme.svg) | Black outlined logo for light surfaces. |
| [logo-dark-theme.svg](logo-dark-theme.svg) | Off-white outlined logo for dark surfaces. |

Keep the selected SVG geometry, gradients and aspect ratios. Logos have transparent backgrounds and contain outlined lettering; they do not require an installed font. Internal SVG IDs use unversioned names.

Application typography uses **Poppins from Google Fonts**. Regular (400), Medium (500), SemiBold (600) and Bold (700) are bundled in [src/assets/fonts/poppins](../../src/assets/fonts/poppins), including Latin and Devanagari glyphs, so both application windows work offline. Shared font declarations live in [src/styles.css](../../src/styles.css). Code and abbreviation fields retain their monospace styling. See the [font license](../../public/licenses/Poppins-OFL.txt) and [third-party notices](../../THIRD_PARTY_NOTICES.md). The license is included in the desktop build.

To regenerate the native application icons with the pinned Bun executable on PATH:

```powershell
bun run tauri icon assets/brand/icon-with-bg.svg --output .tools/brand-icons
```

Copy the generated files matching the existing `src-tauri/icons` filenames into that directory. Mobile icon outputs are not used by this desktop application. The tray uses Tauri's default application icon. Both HTML entries reference the same SVG for their favicon, and Vite bundles it locally.
