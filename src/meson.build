manifest_path = meson.project_source_root() / 'Cargo.toml'
cargo_home = meson.project_build_root() / 'cargo-home'
cargo_target_dir = meson.project_build_root() / 'target'

cargo_build = custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: bindir,
  command: [
    cargo_script,
    meson.project_build_root(),
    meson.project_source_root(),
    '@OUTPUT@',
    profile,
    meson.project_name(),
  ]
)

test(
  'cargo-test',
  cargo,
  args: [
    'test',
    '--manifest-path=@0@'.format(manifest_path),
    '--target-dir=@0@'.format(cargo_target_dir),
    '--',
    '--nocapture',
  ],
  env: [
    'RUST_BACKTRACE=1',
    'CARGO_HOME=@0@'.format(cargo_home),
    'PATH=/app/bin:/usr/bin:/usr/lib/sdk/rust-stable/bin',
  ],
  timeout: 300, # give cargo more time
)

test(
  'cargo-clippy',
  cargo,
  args: [
    'clippy',
    '--manifest-path=@0@'.format(manifest_path),
    '--target-dir=@0@'.format(cargo_target_dir),
  ],
  env: [
    'CARGO_HOME=@0@'.format(cargo_home),
    'PATH=/app/bin:/usr/bin:/usr/lib/sdk/rust-stable/bin',
  ],
  timeout: 300, # give cargo more time
)
