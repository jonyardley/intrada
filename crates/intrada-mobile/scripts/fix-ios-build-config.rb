#!/usr/bin/env ruby
# frozen_string_literal: true

# fix-ios-build-config.rb
#
# Patches the Tauri-generated `gen/apple/project.yml` to work around a
# real iOS build failure (#476): xcodegen treats Tauri's `Externals/`
# directory as a Sources path, which causes Xcode to generate
# `CpResource` build phases for both `Externals/arm64/debug/libapp.a`
# and `Externals/arm64/release/libapp.a` — both targeting the same
# destination `Intrada.app/libapp.a`. xcodebuild errors out with:
#
#   error: Multiple commands produce '...Intrada.app/libapp.a'
#   warning: duplicate output file '...Intrada.app/libapp.a' on task: CpResource
#
# `Externals/` is meant to hold the staged Rust core library for linking
# (referenced via `LIBRARY_SEARCH_PATHS` in the same project.yml). It's
# not supposed to be in any build phase — the linker finds libapp.a via
# search paths, and the resulting binary is embedded as a `framework:`
# dependency on the main target. xcodegen's default of putting source
# paths into the Sources phase is wrong for this directory.
#
# Fix: set `buildPhase: none` on the `Externals` source entry so
# xcodegen excludes it from all build phases. The directory still
# appears in the project tree (Xcode needs to know it exists), but
# nothing inside is added to a copy / compile / resource step.
#
# When to run: once after `cargo tauri ios init` (and again any time you
# regenerate gen/apple/). Idempotent — safe to re-run.
#
# Upstream: this should ideally land in cargo-mobile2's project.yml.hbs
# template. Until then we patch locally.
#
# Tracks: #476.

require 'pathname'
require 'yaml'

# ── Paths ─────────────────────────────────────────────────────────────
MOBILE_ROOT = Pathname.new(__dir__).parent.expand_path
GEN_DIR     = MOBILE_ROOT.join('src-tauri/gen/apple')
PROJECT_YML = GEN_DIR.join('project.yml')

# ── Pre-flight ────────────────────────────────────────────────────────
unless PROJECT_YML.exist?
  abort <<~ERR
    ERROR: #{PROJECT_YML} not found.
    Run `cargo tauri ios init` from #{MOBILE_ROOT.join('src-tauri')} first.
  ERR
end

unless system('which', 'xcodegen', out: File::NULL, err: File::NULL)
  abort <<~ERR
    ERROR: xcodegen not found on PATH.
    Tauri's iOS workflow depends on xcodegen — install with:
      brew install xcodegen
  ERR
end

# ── Mutate ────────────────────────────────────────────────────────────
project = YAML.load_file(PROJECT_YML)

mutated = false
(project['targets'] || {}).each do |name, target|
  sources = target['sources']
  next unless sources.is_a?(Array)

  sources.each do |src|
    next unless src.is_a?(Hash) && src['path'] == 'Externals'
    next if src['buildPhase'] == 'none'

    src['buildPhase'] = 'none'
    mutated = true
    puts "Patched Externals -> buildPhase: none in target: #{name}"
  end
end

# ── Info.plist properties ────────────────────────────────────────────
# Ensure production-ready plist values survive `cargo tauri ios init`.
REQUIRED_PLIST = {
  'UIInterfaceOrientationPortrait' => :orientation,
  'CFBundleDisplayName'            => 'Intrada',
  'UIUserInterfaceStyle'           => 'Dark',
  'UIStatusBarStyle'               => 'UIStatusBarStyleLightContent',
  'UIViewControllerBasedStatusBarAppearance' => false,
  'ITSAppUsesNonExemptEncryption'  => false,
  'UIRequiresFullScreen'           => true,
  'UIBackgroundModes'              => ['audio'],
  'NSCameraUsageDescription'       => 'Intrada uses your camera to take photos of your music.',
  'NSPhotoLibraryUsageDescription' => 'Intrada accesses your photo library to add images to your pieces.',
  'UILaunchScreen'                 => {},
}.freeze

(project['targets'] || {}).each do |name, target|
  props = target.dig('info', 'properties')
  next unless props.is_a?(Hash)

  # Lock iPhone to portrait only
  orientations = props['UISupportedInterfaceOrientations']
  if orientations.is_a?(Array) && orientations != ['UIInterfaceOrientationPortrait']
    props['UISupportedInterfaceOrientations'] = ['UIInterfaceOrientationPortrait']
    mutated = true
    puts "Locked iPhone to portrait-only in target: #{name}"
  end

  # Apply remaining plist properties
  REQUIRED_PLIST.each do |key, value|
    next if key == 'UIInterfaceOrientationPortrait' # handled above
    next if props[key] == value

    props[key] = value
    mutated = true
    puts "Set #{key} in target: #{name}"
  end
end

# ── PATH fix for Xcode builds ────────────────────────────────────────
# Xcode doesn't inherit the user's shell PATH, so `cargo` is not found
# when building via the Run button. Prepend a PATH export to the
# "Build Rust Code" preBuildScript.
(project['targets'] || {}).each do |name, target|
  scripts = target.dig('preBuildScripts')
  next unless scripts.is_a?(Array)

  scripts.each do |script|
    next unless script.is_a?(Hash) && script['name'] == 'Build Rust Code'
    body = script['script'].to_s
    next if body.include?('.cargo/bin')

    script['script'] = "export PATH=\"$HOME/.cargo/bin:/opt/homebrew/bin:$PATH\"\n#{body}"
    mutated = true
    puts "Patched Build Rust Code script with PATH export in target: #{name}"
  end
end

if !mutated
  puts 'OK: nothing to patch. Externals already fixed and PATH already set.'
  exit 0
end

PROJECT_YML.write(project.to_yaml)
puts "Wrote #{PROJECT_YML.relative_path_from(MOBILE_ROOT)}"

# ── Re-run xcodegen ───────────────────────────────────────────────────
puts 'Re-running xcodegen to regenerate .xcodeproj...'
ok = Dir.chdir(GEN_DIR) do
  system('xcodegen', 'generate', '--no-env', '--spec', 'project.yml')
end

abort 'ERROR: xcodegen generate failed. Check the output above.' unless ok

puts ''
puts 'Done. Try the iOS build again:'
puts "  cd #{MOBILE_ROOT.join('src-tauri')} && cargo tauri ios build --target aarch64"
