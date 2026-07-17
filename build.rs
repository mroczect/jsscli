use std::process::Command;

fn run_cmd(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
}

fn main() {
    let git_hash =
        run_cmd("git", &["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "unknown".into());
    let git_full_hash = run_cmd("git", &["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".into());
    let git_branch =
        run_cmd("git", &["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_else(|| "unknown".into());

    let git_dirty = Command::new("git")
        .args(["diff", "--quiet"])
        .status()
        .map(|s| !s.success())
        .unwrap_or(true);
    let dirty_state = if git_dirty { "dirty" } else { "clean" };

    let git_commit_count =
        run_cmd("git", &["rev-list", "--count", "HEAD"]).unwrap_or_else(|| "0".into());
    let git_remote_url = run_cmd("git", &["config", "--get", "remote.origin.url"])
        .unwrap_or_else(|| "unknown".into());

    let local_now = chrono::Local::now();
    let utc_now = chrono::Utc::now();
    let build_hash = local_now.format("%Y%m%d%H%M%S").to_string();
    let timestamp_local = local_now.format("%d/%m/%Y | %H:%M:%S").to_string();
    let timestamp_utc = utc_now.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let profile = std::env::var("PROFILE").unwrap_or_default();
    let hostname = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "unknown".into());

    let rustc_version = run_cmd("rustc", &["--version"]).unwrap_or_else(|| "unknown".into());
    let cargo_version = run_cmd("cargo", &["--version"]).unwrap_or_else(|| "unknown".into());

    let pkg_name = env!("CARGO_PKG_NAME").to_string();
    let pkg_version = env!("CARGO_PKG_VERSION").to_string();
    let pkg_description = env!("CARGO_PKG_DESCRIPTION").to_string();
    let pkg_repository = env!("CARGO_PKG_REPOSITORY").to_string();

    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
    println!("cargo:rustc-env=GIT_FULL_HASH={}", git_full_hash);
    println!("cargo:rustc-env=GIT_BRANCH={}", git_branch);
    println!("cargo:rustc-env=GIT_DIRTY={}", dirty_state);
    println!("cargo:rustc-env=GIT_COMMIT_COUNT={}", git_commit_count);
    println!("cargo:rustc-env=GIT_REMOTE_URL={}", git_remote_url);

    println!("cargo:rustc-env=BUILD_HASH={}", build_hash);
    println!("cargo:rustc-env=BUILD_TIMESTAMP_LOCAL={}", timestamp_local);
    println!("cargo:rustc-env=BUILD_TIMESTAMP_UTC={}", timestamp_utc);

    println!("cargo:rustc-env=TARGET_OS={}", target_os);
    println!("cargo:rustc-env=TARGET_ARCH={}", target_arch);
    println!("cargo:rustc-env=TARGET_ENV={}", target_env);
    println!("cargo:rustc-env=PROFILE={}", profile);
    println!("cargo:rustc-env=BUILD_HOSTNAME={}", hostname);

    println!("cargo:rustc-env=RUSTC_VERSION={}", rustc_version);
    println!("cargo:rustc-env=CARGO_VERSION={}", cargo_version);

    println!("cargo:rustc-env=PKG_NAME={}", pkg_name);
    println!("cargo:rustc-env=PKG_VERSION={}", pkg_version);
    println!("cargo:rustc-env=PKG_DESCRIPTION={}", pkg_description);
    println!("cargo:rustc-env=PKG_REPOSITORY={}", pkg_repository);

    let changelog_json = run_cmd("git", &[
        "log",
        "--pretty=format:{\"hash\":\"%h\",\"author\":\"%an\",\"date\":\"%ad\",\"message\":\"%s\"}",
        "--date=short",
        "-n", "50"
    ]).unwrap_or_else(|| "".to_string());

    let changelog_array = if changelog_json.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", changelog_json.replace("}\n{", "},{"))
    };

    println!("cargo:rustc-env=CHANGELOG_JSON={}", changelog_array);
}
