// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! What a front end holds: every value that configures a call, in one
//! [`Settings`], which reads and writes JSON with the feature `serialize`,
//! and the [`Clock`] it applies through its stop, which the library never
//! reads.

use crate::export::Styles;
use crate::search::{Jobs, Options, batch};
use crate::{Limits, ViewOptions, ordinary};

/// What a front end with a clock applies through its stop, each counted in
/// milliseconds from the front end's start; the library reads no clock.
/// `None` is no limit.
///
/// # JSON
///
/// With the feature `serialize` an object of the fields, each a number or
/// `null`; a missing one takes its default and a misspelt one is refused.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    serde(default, deny_unknown_fields)
)]
pub struct Clock {
    /// How long a search may run, or a call that holds one.
    #[cfg_attr(feature = "serialize", serde(with = "crate::serialize::exact"))]
    pub time_limit_ms: Option<u64>,
    /// How long a race searches on one thread before a pool of the other
    /// threads searches beside it; `None` for no pool.
    #[cfg_attr(feature = "serialize", serde(with = "crate::serialize::exact"))]
    pub pool_after_ms: Option<u64>,
    /// How long a whole batch may run.
    #[cfg_attr(feature = "serialize", serde(with = "crate::serialize::exact"))]
    pub batch_time_limit_ms: Option<u64>,
}

impl Clock {
    /// The default time limit: two seconds.
    pub const DEFAULT_TIME_LIMIT_MS: u64 = 2000;
    /// The default time a race searches alone: a tenth of a second.
    pub const DEFAULT_POOL_AFTER_MS: u64 = 100;

    /// Returns the clock with another [`time_limit_ms`](Self::time_limit_ms).
    #[must_use]
    pub const fn with_time_limit_ms(self, time_limit_ms: Option<u64>) -> Self {
        Self {
            time_limit_ms,
            ..self
        }
    }

    /// Returns the clock with another [`pool_after_ms`](Self::pool_after_ms).
    #[must_use]
    pub const fn with_pool_after_ms(self, pool_after_ms: Option<u64>) -> Self {
        Self {
            pool_after_ms,
            ..self
        }
    }

    /// Returns the clock with another
    /// [`batch_time_limit_ms`](Self::batch_time_limit_ms).
    #[must_use]
    pub const fn with_batch_time_limit_ms(self, batch_time_limit_ms: Option<u64>) -> Self {
        Self {
            batch_time_limit_ms,
            ..self
        }
    }
}

impl Default for Clock {
    /// [`DEFAULT_TIME_LIMIT_MS`](Self::DEFAULT_TIME_LIMIT_MS),
    /// [`DEFAULT_POOL_AFTER_MS`](Self::DEFAULT_POOL_AFTER_MS), and no limit
    /// on a batch.
    fn default() -> Self {
        Self {
            time_limit_ms: Some(Self::DEFAULT_TIME_LIMIT_MS),
            pool_after_ms: Some(Self::DEFAULT_POOL_AFTER_MS),
            batch_time_limit_ms: None,
        }
    }
}

/// Everything that configures a front end's calls, in one value: the
/// clock, the limits, the search's options, the view of a derivation, the
/// options of every output format, the batch's and ordinary logic's.
///
/// `Settings::default()` is the command's behaviour: no copy bound (a
/// front end with a clock deepens until its time limit), every thread
/// the machine runs, the clock's two seconds and a tenth, and the default
/// limits. It differs from [`search::Options::default()`](Options), which
/// keeps a copy bound of three, because a library call without a stop
/// must end.
///
/// # JSON
///
/// With the feature `serialize` an object with a key per field, each the
/// JSON of that value; a missing key, or a missing field of one, takes
/// its default, and a misspelt one is refused.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    serde(default, deny_unknown_fields)
)]
pub struct Settings {
    /// What the front end applies through its stop.
    pub clock: Clock,
    /// The resources a call may use.
    pub limits: Limits,
    /// The search's options.
    pub search: Options,
    /// How a derivation is shown.
    pub view: ViewOptions,
    /// The options of every output format.
    pub styles: Styles,
    /// How a batch shares the machine.
    pub batch: batch::Options,
    /// How an ordinary sequent is decided.
    pub ordinary: ordinary::Options,
}

impl Default for Settings {
    /// The command's behaviour: see [`Settings`].
    fn default() -> Self {
        Self {
            clock: Clock::default(),
            limits: Limits::default(),
            search: Options::default().with_copies(None).with_jobs(Jobs::Auto),
            view: ViewOptions::default(),
            styles: Styles::default(),
            batch: batch::Options::default(),
            ordinary: ordinary::Options::default(),
        }
    }
}

impl Settings {
    /// Returns the settings with another [`clock`](Self::clock).
    #[must_use]
    pub fn with_clock(self, clock: Clock) -> Self {
        Self { clock, ..self }
    }

    /// Returns the settings with another [`limits`](Self::limits).
    #[must_use]
    pub fn with_limits(self, limits: Limits) -> Self {
        Self { limits, ..self }
    }

    /// Returns the settings with another [`search`](Self::search).
    #[must_use]
    pub fn with_search(self, search: Options) -> Self {
        Self { search, ..self }
    }

    /// Returns the settings with another [`view`](Self::view).
    #[must_use]
    pub fn with_view(self, view: ViewOptions) -> Self {
        Self { view, ..self }
    }

    /// Returns the settings with another [`styles`](Self::styles).
    #[must_use]
    pub fn with_styles(self, styles: Styles) -> Self {
        Self { styles, ..self }
    }

    /// Returns the settings with another [`batch`](Self::batch).
    #[must_use]
    pub fn with_batch(self, batch: batch::Options) -> Self {
        Self { batch, ..self }
    }

    /// Returns the settings with another [`ordinary`](Self::ordinary).
    #[must_use]
    pub fn with_ordinary(self, ordinary: ordinary::Options) -> Self {
        Self { ordinary, ..self }
    }
}
