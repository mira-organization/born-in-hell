#![coverage(off)]

use tiled::{Color, ObjectShape, PropertyValue};

/// Convenience accessors for `tiled::PropertyValue` that return typed values
/// or `None` when the underlying variant does not match. Includes `*_or`
/// helpers to provide defaults without branching.
pub trait PropertyValueExt {
    /// Returns `Some(bool)` if this is `BoolValue`, otherwise `None`.
    fn as_bool(&self) -> Option<bool>;
    /// Returns `Some(f32)` if this is `FloatValue`, otherwise `None`.
    fn as_f32(&self) -> Option<f32>;
    /// Returns `Some(i32)` if this is `IntValue`, otherwise `None`.
    fn as_i32(&self) -> Option<i32>;
    /// Returns `Some(u32)` if this is `ObjectValue` (object id), otherwise `None`.
    fn as_u32(&self) -> Option<u32>;
    /// Returns `Some(&str)` if this is `StringValue` or `FileValue`, otherwise `None`.
    fn as_str(&self) -> Option<&str>;
    /// Returns `Some(Color)` if this is `ColorValue`, otherwise `None`.
    fn as_color(&self) -> Option<Color>;

    /// Returns the contained bool or `default` if not a `BoolValue`.
    fn bool_or(&self, default: bool) -> bool { self.as_bool().unwrap_or(default) }
    /// Returns the contained f32 or `default` if not a `FloatValue`.
    fn f32_or(&self, default: f32) -> f32 { self.as_f32().unwrap_or(default) }
    /// Returns the contained i32 or `default` if not an `IntValue`.
    fn i32_or(&self, default: i32) -> i32 { self.as_i32().unwrap_or(default) }
    /// Returns the contained u32 (object id) or `default` if not an `ObjectValue`.
    fn u32_or(&self, default: u32) -> u32 { self.as_u32().unwrap_or(default) }
    /// Returns the contained `&str` or `default` if not a string/file value.
    ///
    /// # Parameters
    /// * `default` – Fallback string slice to return when no string is present.
    fn str_or<'a>(&'a self, default: &'a str) -> &'a str { self.as_str().unwrap_or(default) }
}

impl PropertyValueExt for PropertyValue {
    fn as_bool(&self) -> Option<bool> {
        match self { PropertyValue::BoolValue(v) => Some(*v), _ => None }
    }
    fn as_f32(&self) -> Option<f32> {
        match self { PropertyValue::FloatValue(v) => Some(*v), _ => None }
    }
    fn as_i32(&self) -> Option<i32> {
        match self { PropertyValue::IntValue(v) => Some(*v), _ => None }
    }
    fn as_u32(&self) -> Option<u32> {
        match self { PropertyValue::ObjectValue(v) => Some(*v), _ => None }
    }
    fn as_str(&self) -> Option<&str> {
        match self {
            PropertyValue::StringValue(s) | PropertyValue::FileValue(s) => Some(s.as_str()),
            _ => None
        }
    }
    fn as_color(&self) -> Option<Color> {
        match self { PropertyValue::ColorValue(c) => Some(*c), _ => None }
    }
}

/// Utility accessors for `tiled::ObjectShape` dimensions. Non-rect shapes
/// return `0.0` for width/height by design.
pub trait ObjectShapeExt {
    /// Returns the width if the shape is `Rect`, otherwise `0.0`.
    fn get_width(&self) -> f32;
    /// Returns the height if the shape is `Rect`, otherwise `0.0`.
    fn get_height(&self) -> f32;
}

impl ObjectShapeExt for ObjectShape {
    fn get_width(&self) -> f32 {
        match self {
            ObjectShape::Rect { width, .. } => *width,
            _  => 0.0
        }
    }

    fn get_height(&self) -> f32 {
        match self {
            ObjectShape::Rect { height, .. } => *height,
            _  => 0.0
        }
    }
}
