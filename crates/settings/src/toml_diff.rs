use toml::{map::Map, value::Array, Value};

// todo: toml to reflect value......
// user modify => modify user config
// user config diff base config to get diff toml and to save
//

pub fn toml_diff<S>(base: &S, over: &S) -> Result<S, toml::de::Error>
where
    S: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    let base = toml::to_string(base).unwrap();
    let diff = toml::to_string(over).unwrap();
    let over = toml::to_string(over).unwrap();

    let base_map = base.parse::<toml::Table>().unwrap();
    let over_map = over.parse::<toml::Table>().unwrap();
    let mut diff_map = diff.parse::<toml::Table>().unwrap();

    toml_map_diff(&base_map, &over_map, &mut diff_map);

    diff_map.try_into()
}

fn toml_array_diff(base_array: &Array, override_array: &Array, diff_array: &mut Array) {
    base_array.iter().for_each(|v| {
        if let Some(override_index) = override_array.iter().position(|o| o == v) {
            let Some(ov) = override_array.get(override_index) else {
                return;
            };

            if v == ov {
                diff_array.retain(|d| d != v);
            } else {
                match v {
                    Value::Table(table) => {
                        let override_table = ov.as_table().unwrap();
                        let diff_index = diff_array.iter().position(|d| d == v).unwrap();
                        let diff_table = diff_array
                            .get_mut(diff_index)
                            .unwrap()
                            .as_table_mut()
                            .unwrap();
                        toml_map_diff(table, override_table, diff_table);
                    }
                    Value::Array(array) => {
                        let override_array = ov.as_array().unwrap();
                        let diff_index = diff_array.iter().position(|d| d == v).unwrap();
                        let diff_array = diff_array
                            .get_mut(diff_index)
                            .unwrap()
                            .as_array_mut()
                            .unwrap();
                        toml_array_diff(array, override_array, diff_array);
                    }
                    _ => {}
                }
            }
        }
    });
}

fn toml_map_diff(
    base_map: &Map<String, Value>,
    override_map: &Map<String, Value>,
    diff_map: &mut Map<String, Value>,
) {
    base_map.iter().for_each(|(k, v)| {
        if let Some(ov) = override_map.get(k) {
            if v == ov {
                diff_map.remove(k);
            } else {
                match v {
                    Value::Table(table) => {
                        let override_table = ov.as_table().unwrap();
                        let diff_table = diff_map.get_mut(k).unwrap().as_table_mut().unwrap();
                        toml_map_diff(table, override_table, diff_table);
                    }
                    Value::Array(array) => {
                        let override_array = ov.as_array().unwrap();
                        let diff_array = diff_map.get_mut(k).unwrap().as_array_mut().unwrap();
                        toml_array_diff(array, override_array, diff_array);
                    }
                    _ => {}
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use crate::toml_diff::toml_diff;

    #[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
    struct TestSettings {
        a: Option<u32>,
        b: Option<String>,
    }

    #[test]
    fn test_toml_diff() {
        let at = TestSettings {
            a: Some(1),
            b: Some("a".to_string()),
        };

        let bt = TestSettings {
            a: Some(1),
            b: Some("b".to_string()),
        };

        let table = toml_diff(&at, &bt);
        assert_eq!(
            table.unwrap(),
            TestSettings {
                a: None,
                b: Some("b".to_string())
            }
        );
    }
}
