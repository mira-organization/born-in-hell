use tiled::{Color, ObjectShape, PropertyValue};

pub trait PropertyValueExt {
    fn as_bool(&self) -> Option<bool>;
    fn as_f32(&self) -> Option<f32>;
    fn as_i32(&self) -> Option<i32>;
    fn as_u32(&self) -> Option<u32>;
    fn as_str(&self) -> Option<&str>;
    fn as_color(&self) -> Option<Color>;

    fn bool_or(&self, default: bool) -> bool { self.as_bool().unwrap_or(default) }
    fn f32_or(&self, default: f32) -> f32 { self.as_f32().unwrap_or(default) }
    fn i32_or(&self, default: i32) -> i32 { self.as_i32().unwrap_or(default) }
    fn u32_or(&self, default: u32) -> u32 { self.as_u32().unwrap_or(default) }
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

pub trait ObjectShapeExt {
    fn get_width(&self) -> f32;

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
