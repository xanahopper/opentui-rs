//! `opentui-core` — Higher-level UI abstractions built on `opentui_rust`
//!
//! This crate provides layout, widgets, and scrollable views on top of the
//! `opentui_rust` rendering engine. It fills the gap between the low-level
//! buffer/cell API and a full application UI.
//!
//! # Architecture
//!
//! ```text
//! Application
//!     |
//!     v
//! +---------------------------+
//! | opentui-core (this crate) |
//! |  layout  widget  scroll   |
//! |  list    event   theme    |
//! +---------------------------+
//!     |
//!     v
//! +---------------------------+
//! | opentui_rust (engine)     |
//! |  buffer  cell  renderer   |
//! |  input   text  terminal   |
//! +---------------------------+
//!     |
//!     v
//!   Terminal (stdout)
//! ```
//!
//! # Modules
//!
//! - `layout`: Flexbox and grid layout via [Taffy](https://github.com/DioxusLabs/taffy)
//! - `widget`: Base widget trait, widget tree, rendering context
//! - `scroll`: ScrollView with scrollbar, viewport culling
//! - `list`: Virtual list with lazy rendering for large datasets
//! - `event`: Higher-level event dispatch, focus management, hit testing
//! - `theme`: Theme definition and resolution

#![warn(unsafe_code)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::option_if_let_else)]

pub mod event;
pub mod keybinding;
pub mod layout;
pub mod list;
pub mod render_command;
pub mod scroll;
pub mod theme;
pub mod widget;
pub mod widgets;

pub use layout::LayoutEngine;
pub use widget::{RenderContext, Widget, WidgetId, WidgetTree};
