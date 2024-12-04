// SPDX-FileCopyrightText: 2024 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use once_cell::sync::Lazy;
use std::path::PathBuf;

/// Default value for the port part of [`crate::Config::addr`].
pub const DEFAULT_PORT: u16 = 3000;
/// Default value for the address part of [`crate::Config::addr`].
pub const DEFAULT_ADDRESS: &str = "127.0.0.1";
/// Default value for [`crate::Config::cache_root`].
pub static DEFAULT_CACHE_ROOT: Lazy<PathBuf> =
    Lazy::new(|| dirs::cache_dir().unwrap().join(clap::crate_name!()));
