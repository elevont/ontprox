// SPDX-FileCopyrightText: 2024 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;
use std::sync::LazyLock;

/// Default value for the port part of [`crate::Config::addr`].
pub const DEFAULT_PORT: u16 = 3000;
/// Default value for [`crate::Config::timeout`].
pub const DEFAULT_TIMEOUT: u16 = 30;
/// Default value for the address part of [`crate::Config::addr`].
pub const DEFAULT_ADDRESS: &str = "127.0.0.1";
/// Default value for [`crate::Config::cache_root`].
pub static DEFAULT_CACHE_ROOT: LazyLock<PathBuf> =
    LazyLock::new(|| dirs::cache_dir().unwrap().join(clap::crate_name!()));
