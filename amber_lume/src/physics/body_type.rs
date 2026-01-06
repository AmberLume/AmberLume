use alpaca::data::common::scene_data::BodyTypeData;

#[derive(Debug, Eq, PartialEq)]
pub enum BodyType {
    Static,
    Kinematic,
    Dynamic,
}

impl BodyType {
    pub fn from_data(body_type_data: &BodyTypeData) -> Self {
        match body_type_data {
            BodyTypeData::Static => BodyType::Static,
            BodyTypeData::Kinematic => BodyType::Kinematic,
            BodyTypeData::Dynamic => BodyType::Dynamic,
        }
    }
}

