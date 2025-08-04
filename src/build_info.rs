pub struct BuildInfo {
  pub git_hash: &'static str,
  pub git_dirty: bool,
  pub profile: &'static str,
}

pub const BUILD_INFO: BuildInfo = BuildInfo {
  git_hash: env!("GIT_HASH"),
  git_dirty: matches!(env!("GIT_DIRTY").as_bytes(), b"1"),
  profile: env!("BUILD_PROFILE"),
};
