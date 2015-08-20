#![feature(drain)]

#[cfg(test)]
extern crate assert;

#[macro_use]
extern crate log;

extern crate fractal;
extern crate options;
extern crate probability;
extern crate random;
extern crate sql;
extern crate sqlite;
extern crate temperature;
extern crate threed_ice;
extern crate toml;

use std::{error, fmt};

macro_rules! raise(
    ($message:expr) => (return Err(Box::new(::ErrorString($message.to_string()))));
    ($($arg:tt)*) => (return Err(Box::new(::ErrorString(format!($($arg)*)))));
);

macro_rules! ok(
    ($result:expr) => (match $result {
        Ok(result) => result,
        Err(error) => return Err(Box::new(error)),
    });
);

macro_rules! some(
    ($option:expr, $($arg:tt)*) => (match $option {
        Some(value) => value,
        _ => raise!($($arg)*),
    });
);

macro_rules! path(
    ($config:ident, $destination:expr) => ({
        let path = some!($config.get::<String>("path"), "the path to {} is missing", $destination);
        let mut path = ::std::path::PathBuf::from(path);
        if path.is_relative() {
            if let Some(ref root) = $config.get::<::std::path::PathBuf>("root") {
                path = root.join(path);
            }
        }
        if ::std::fs::metadata(&path).is_err() {
            raise!("the file {:?} does not exist", &path);
        }
        path
    });
);

macro_rules! rewrite(
    ($header:tt [$($member:tt)*] {}) => (
        rewrite!(output $header [$($member)*]);
    );
    ($header:tt [$($member:tt)*] { pub $name:ident: $kind:ty, $($t:tt)* }) => (
        rewrite!($header [$($member)* pub $name: $kind,] { $($t)* });
    );
    ($header:tt [$($member:tt)*] { $name:ident: $kind:ty, $($t:tt)* }) => (
        rewrite!($header [$($member)* $name: $kind,] { $($t)* });
    );
    (output [$($chunk:tt)*] [$($member:tt)*]) => (
        itemize!(
            $($chunk)* {
                $($member)*
            }
        );
    );
);

macro_rules! rc(
    (
        $(#[$attr:meta])*
        pub struct $outer:ident($inner:ident) $body:tt
    ) => (
        $(#[$attr])*
        pub struct $outer(::std::rc::Rc<$inner>);

        rewrite! {
            [$(#[$attr])* pub struct $inner] [] $body
        }

        impl ::std::ops::Deref for $outer {
            type Target = $inner;

            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    );
    ($outer:ident($inner:ident { $($field:ident: $value:expr),* })) => (
        rc!($outer($inner { $($field: $value,)* }))
    );
    ($outer:ident($inner:ident { $($field:ident: $value:expr,)* })) => (
        $outer(::std::rc::Rc::new($inner { $($field: $value,)* }))
    );
);

macro_rules! order {
    ($name:ident($field:tt) ascending) => (itemize! {
        impl ::std::cmp::Eq for $name {
        }

        impl ::std::cmp::Ord for $name {
            fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                if self.$field < other.$field {
                    ::std::cmp::Ordering::Less
                } else if self.$field > other.$field {
                    ::std::cmp::Ordering::Greater
                } else {
                    ::std::cmp::Ordering::Equal
                }
            }
        }

        impl ::std::cmp::PartialEq for $name {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                self.$field == other.$field
            }
        }

        impl ::std::cmp::PartialOrd for $name {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }
    });
    ($name:ident($field:tt) descending) => (itemize! {
        impl ::std::cmp::Eq for $name {
        }

        impl ::std::cmp::Ord for $name {
            fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                if self.$field < other.$field {
                    ::std::cmp::Ordering::Greater
                } else if self.$field > other.$field {
                    ::std::cmp::Ordering::Less
                } else {
                    ::std::cmp::Ordering::Equal
                }
            }
        }

        impl ::std::cmp::PartialEq for $name {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                self.$field == other.$field
            }
        }

        impl ::std::cmp::PartialOrd for $name {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }
    });
}

macro_rules! itemize(($($blob:item)*) => ($($blob)*));

mod config;
mod platform;
mod profile;
mod schedule;
mod system;
mod traffic;
mod workload;

pub use config::Config;
pub use platform::Platform;
pub use profile::Profile;
pub use system::{Increment, Job, System};

pub type Error = Box<std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

pub struct ErrorString(pub String);

pub type Source = random::Default;

impl fmt::Debug for ErrorString {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for ErrorString {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl error::Error for ErrorString {
    #[inline]
    fn description(&self) -> &str {
        &self.0
    }
}
