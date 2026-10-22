//! The internal "Language of Geometry" for the UI engine.
//!
//! This module centralizes the foundational data structures that define
//! space, color, and alignment. Unlike the `widget` module, which
//! contains "Data-Driven Entities" with logic and state, the `types` module
//! provides the "Passive Data" used to describe layout intent.
//!
//! ### The "POD" Philosophy (Plain Old Data)
//! Types in this module are designed to be lightweight, highly clonable,
//! and strictly separated from rendering side-effects. This ensures
//! that a `Widget` can be configured using these types safely before
//! the heavy lifting of the compilation phase begins.
//!
//! ### Layout Interoperability
//! By standardizing primitives like [`Extent2D`] and [`Spacing`], the
//! engine allows disparate widgets—such as a `Text` element and an
//! `Image`—to exist within the same `FlexContainer` using a shared
//! coordinate system.

pub mod alignment;
pub mod border;
pub mod color;
pub mod direction;
pub mod dirty_flags;
pub mod extent;
pub mod identifiers;
pub mod input_behavior;
pub mod offset;
pub mod point;
pub mod shader_builder;
pub mod spacing;
pub mod style;

pub use alignment::{Alignment, Position};
pub use border::Border;
pub use color::{Bgra, Color, LinearGradient};
pub use direction::Direction;
pub use dirty_flags::DirtyFlags;
pub use extent::Extent;
pub use identifiers::{WidgetId, WidgetKey};
pub use input_behavior::InputBehavior;
pub use offset::Offset;
pub use point::Point;
pub use shader_builder::{ShaderBuilder, ShaderBuilderError, UniformValue};
pub use spacing::Spacing;
