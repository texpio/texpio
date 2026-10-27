# Texpio brand assets

The approved [brand toolkit](../brand-kit/README.md) and its [guide](../brand-kit/brand-guide.html) are the source for this identity. The selected [icon master](../brand-kit/logos/icon-master-10-percent.svg) keeps the original canvas and reduces the connected-dot illustration by 10%; the toolkit's app icon, favicon and logo lockups use that geometry. The outlined wordmark does not depend on an installed font.

| App asset | Use |
| --- | --- |
| [icon-with-bg.svg](icon-with-bg.svg) | Ink rounded square and mint mark; source for desktop, installer and tray icons. |
| [icon.svg](icon.svg) | Standalone ink mark on a transparent background. |
| [logo-light-theme.svg](logo-light-theme.svg) | Forest and ink lockup for light surfaces. |
| [logo-dark-theme.svg](logo-dark-theme.svg) | Mint and paper lockup for dark surfaces. |
| [favicon.svg](favicon.svg) | Toolkit favicon geometry, ink on light browser chrome and mint when the browser requests a dark color scheme. |

The shared theme in `src/styles.css` maps ink, paper, forest, mint and related surface colors to semantic tokens. Forest is used for light-theme actions and mint for dark-theme actions; do not use mint as small text on paper. Check contrast for new combinations and states. Preserve the lockup proportions and keep icon and wordmark from the same toolkit.

Application typography uses **Poppins from Google Fonts**. Regular (400), Medium (500), SemiBold (600) and Bold (700) are bundled in [src/assets/fonts/poppins](../../src/assets/fonts/poppins), including Latin and Devanagari glyphs, so both application windows work offline. Shared font declarations live in [src/styles.css](../../src/styles.css). Code and abbreviation fields retain their monospace styling. See the [font license](../../public/licenses/Poppins-OFL.txt) and [third-party notices](../../THIRD_PARTY_NOTICES.md). The license is included in the desktop build.

## Regenerating desktop icons

Use pinned Bun 1.4.4 on PATH and the installed Tauri CLI:

```sh
bun run tauri icon assets/brand/icon-with-bg.svg --output .tools/brand-icons
```

Copy the generated files matching the existing `src-tauri/icons` filenames into that directory. Mobile outputs are not used by this desktop application. The tray uses Tauri's default application icon. Both HTML entries reference the adaptive SVG favicon, which Vite bundles locally. Check the built icon and tray on each target platform; generated files alone do not verify native presentation.
