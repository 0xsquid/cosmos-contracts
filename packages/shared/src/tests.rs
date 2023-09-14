use serde_cw_value::Value;

use crate::util::json_pointer;

#[test]
fn test_json_pointer() {
    let mut obj: Value = serde_json_wasm::from_str(
        r#"
    {
        "address": "obj1_addr_1",
        "addresses": [
            "vec_obj1_addr_1",
            "vec_obj1_addr_2",
            "vec_obj1_addr_3"
        ]
    }
    "#,
    )
    .unwrap();

    let same = json_pointer(&mut obj, "").unwrap().clone();
    assert_eq!(same, obj);

    assert_eq!(json_pointer(&mut obj, "path/to/value"), None);

    let address_value = json_pointer(&mut obj, "/address").unwrap().clone();
    assert_eq!(address_value, Value::String("obj1_addr_1".to_owned()));

    let vec_address_value = json_pointer(&mut obj, "/addresses/1").unwrap().clone();
    assert_eq!(
        vec_address_value,
        Value::String("vec_obj1_addr_2".to_owned())
    );

    let mut obj: Value = serde_json_wasm::from_str(
        r#"
    {
        "objects": [
            {
                "address": "obj1_addr_1"
            },
            {
                "address": "obj1_addr_1"
            }
        ]
    }
    "#,
    )
    .unwrap();

    let object_address_value = json_pointer(&mut obj, "/objects/0/address")
        .unwrap()
        .clone();
    assert_eq!(
        object_address_value,
        Value::String("obj1_addr_1".to_owned())
    );
}
