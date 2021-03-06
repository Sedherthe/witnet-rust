use serde_cbor::value::{from_value, to_value, Value};
use std::{
    collections::{btree_map::BTreeMap, HashMap},
    convert::{TryFrom, TryInto},
    fmt,
};

use crate::{
    error::RadError,
    operators::{identity, map as map_operators, Operable, RadonOpCodes},
    script::RadonCall,
    types::{RadonType, RadonTypes},
};
use witnet_data_structures::radon_report::ReportContext;

pub const RADON_MAP_TYPE_NAME: &str = "RadonMap";

#[derive(Clone, Debug, PartialEq, Default)]
pub struct RadonMap {
    value: HashMap<String, RadonTypes>,
}

impl RadonType<HashMap<String, RadonTypes>> for RadonMap {
    fn value(&self) -> HashMap<String, RadonTypes> {
        self.value.clone()
    }

    fn radon_type_name() -> String {
        RADON_MAP_TYPE_NAME.to_string()
    }
}

impl From<HashMap<String, RadonTypes>> for RadonMap {
    fn from(value: HashMap<String, RadonTypes>) -> Self {
        RadonMap { value }
    }
}

impl From<BTreeMap<String, RadonTypes>> for RadonMap {
    fn from(value: BTreeMap<String, RadonTypes>) -> Self {
        RadonMap {
            value: value.into_iter().collect(),
        }
    }
}

impl TryFrom<Value> for RadonMap {
    type Error = RadError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let error = |_| RadError::Decode {
            from: "cbor::value::Value".to_string(),
            to: RADON_MAP_TYPE_NAME.to_string(),
        };

        let hm = from_value::<HashMap<String, Value>>(value)
            .map_err(error)?
            .iter()
            .filter_map(|(key, value)| match RadonTypes::try_from(value.clone()) {
                Ok(x) => Some((key.clone(), x)),
                Err(_) => None,
            })
            .collect::<HashMap<String, RadonTypes>>();

        Ok(RadonMap::from(hm))
    }
}

impl TryFrom<RadonTypes> for RadonMap {
    type Error = RadError;

    fn try_from(item: RadonTypes) -> Result<Self, Self::Error> {
        if let RadonTypes::Map(rad_map) = item {
            Ok(rad_map)
        } else {
            let value = Value::try_from(item)?;
            value.try_into()
        }
    }
}

impl TryInto<Value> for RadonMap {
    type Error = RadError;

    fn try_into(self) -> Result<Value, Self::Error> {
        let error = || RadError::Encode {
            from: RADON_MAP_TYPE_NAME.to_string(),
            to: "cbor::value::Value".to_string(),
        };

        let map = self
            .value()
            .iter()
            .try_fold(
                BTreeMap::<Value, Value>::new(),
                |mut map, (key, radon_types)| {
                    if let (Ok(key), Ok(value)) = (
                        Value::try_from(key.to_string()),
                        Value::try_from(radon_types.clone()),
                    ) {
                        map.insert(key, value);
                        Some(map)
                    } else {
                        None
                    }
                },
            )
            .ok_or_else(error)?;

        to_value(map).map_err(|_| error())
    }
}

impl fmt::Display for RadonMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({:?})", RADON_MAP_TYPE_NAME, self.value)
    }
}

impl Operable for RadonMap {
    fn operate(&self, call: &RadonCall) -> Result<RadonTypes, RadError> {
        match call {
            (RadonOpCodes::Identity, None) => identity(RadonTypes::from(self.clone())),
            (RadonOpCodes::MapGetArray, Some(args)) => {
                map_operators::get_array(self, args.as_slice()).map(RadonTypes::from)
            }
            (RadonOpCodes::MapGetBoolean, Some(args)) => {
                map_operators::get_boolean(self, args.as_slice()).map(RadonTypes::from)
            }
            (RadonOpCodes::MapGetBytes, Some(args)) => {
                map_operators::get_bytes(self, args.as_slice()).map(RadonTypes::from)
            }
            (RadonOpCodes::MapGetInteger, Some(args)) => {
                map_operators::get_integer(self, args.as_slice()).map(RadonTypes::from)
            }
            (RadonOpCodes::MapGetFloat, Some(args)) => {
                map_operators::get_float(self, args.as_slice()).map(RadonTypes::from)
            }
            (RadonOpCodes::MapGetMap, Some(args)) => {
                map_operators::get_map(self, args.as_slice()).map(RadonTypes::from)
            }
            (RadonOpCodes::MapGetString, Some(args)) => {
                map_operators::get_string(self, args.as_slice()).map(RadonTypes::from)
            }
            (RadonOpCodes::MapKeys, None) => Ok(RadonTypes::from(map_operators::keys(self))),
            (RadonOpCodes::MapValues, None) => Ok(RadonTypes::from(map_operators::values(self))),
            (op_code, args) => Err(RadError::UnsupportedOperator {
                input_type: RADON_MAP_TYPE_NAME.to_string(),
                operator: op_code.to_string(),
                args: args.to_owned(),
            }),
        }
    }

    fn operate_in_context(
        &self,
        call: &RadonCall,
        _context: &mut ReportContext,
    ) -> Result<RadonTypes, RadError> {
        self.operate(call)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::integer::RadonInteger;
    use witnet_data_structures::radon_report::TypeLike;

    #[test]
    fn test_operate_identity() {
        let mut map = HashMap::new();
        let value = RadonTypes::Integer(RadonInteger::from(0));
        map.insert("Zero".to_string(), value);

        let input = RadonMap::from(map.clone());
        let expected = RadonMap::from(map).into();

        let call = (RadonOpCodes::Identity, None);
        let output = input.operate(&call).unwrap();

        assert_eq!(output, expected);
    }

    #[test]
    fn test_operate_unimplemented() {
        let mut map = HashMap::new();
        let value = RadonTypes::Integer(RadonInteger::from(0));
        map.insert("Zero".to_string(), value);

        let input = RadonMap::from(map);

        let call = (RadonOpCodes::Fail, None);
        let result = input.operate(&call);

        assert!(if let Err(_error) = result {
            true
        } else {
            false
        });
    }

    #[test]
    fn test_try_into() {
        let mut map = HashMap::new();
        let value = RadonTypes::Integer(RadonInteger::from(0));
        map.insert("Zero".to_string(), value);
        let input = RadonMap::from(map);

        let result = RadonTypes::from(input).encode().unwrap();

        let expected_vec: Vec<u8> = vec![161, 100, 90, 101, 114, 111, 0];

        assert_eq!(result, expected_vec);
    }

    #[test]
    fn test_try_from() {
        let slice: &[u8] = &[161, 100, 90, 101, 114, 111, 0];

        let result = RadonTypes::try_from(slice).unwrap();

        let mut map = HashMap::new();
        let value = RadonTypes::Integer(RadonInteger::from(0));
        map.insert("Zero".to_string(), value);
        let expected_input = RadonTypes::from(RadonMap::from(map));

        assert_eq!(result, expected_input);
    }

    #[test]
    fn test_operate_map_get() {
        let mut map = HashMap::new();
        let value = RadonTypes::Integer(RadonInteger::from(0));
        map.insert("Zero".to_string(), value);
        let input = RadonMap::from(map);

        let call = (
            RadonOpCodes::MapGetInteger,
            Some(vec![Value::Text(String::from("Zero"))]),
        );
        let result = input.operate(&call).unwrap();

        let expected_value = RadonTypes::Integer(RadonInteger::from(0));

        assert_eq!(result, expected_value);
    }

    #[test]
    fn test_operate_map_get_error() {
        let mut map = HashMap::new();
        let value = RadonTypes::Integer(RadonInteger::from(0));
        map.insert("Zero".to_string(), value);
        let input = RadonMap::from(map);

        let call = (
            RadonOpCodes::MapGetInteger,
            Some(vec![Value::Text(String::from("NotFound"))]),
        );
        let result = input.operate(&call);

        assert!(result.is_err());
    }
}
