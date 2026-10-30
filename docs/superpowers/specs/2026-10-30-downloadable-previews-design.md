# Downloadable native previews

The user authorized downloadable artifacts for all eight existing CI variants,
pushing the change and running CI so the packages can be downloaded for testing.
Use the existing matrix and current checkout. No app input behavior changes.

Build Windows NSIS installers, ad-hoc signed macOS app bundles archived with tar,
and Linux Debian/AppImage packages. Each Wayland download also contains the
separate helper Debian package and its setup/feasibility notes. These formats use
the existing Tauri bundler and helper packaging instead of introducing a second
packaging system. Bare executable uploads were considered but do not preserve a
usable macOS application bundle. Public release publishing remains in the
existing signed, gated workflow.

Stage only required package files, a preview README, build identity and SHA-256
checksums. Missing or ambiguous packages fail the job. Upload one uniquely named
artifact per OS/backend/architecture with 14-day retention. Expose its download
link in the job summary. Keep signing credentials and release publication out of
the ordinary CI workflow. Mac previews use the ad-hoc identity `-`.

Verify package selection and macOS archive structure with disposable fixtures,
run repository checks, build/restart the Windows development preview as required,
then push and verify all eight uploaded artifacts. Native interactive macOS/Linux
qualification and the strict composition requirements remain pending.
