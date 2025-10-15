use crate::export::{Exporter, FontoSchemaExporter};
use crate::formats::fonto;

#[test]
fn test_deserialize_niso_sts() {
    let schema: fonto::Schema =
        serde_json::from_str(include_str!("../formats/fonto/niso-sts.json")).unwrap();
}

#[test]
fn test_export() {
    let schema = crate::tests::get_compiled_schema();
    let exporter = FontoSchemaExporter::default();
    let fonto_schema = exporter.export_schema(&schema).unwrap();

    fonto_schema
        .save_to_file("./output/tests/whas-test.fonto-schema.json")
        .unwrap();

    assert_eq!(
        fonto_schema.simple_types().len(),
        schema.types_simple().len()
    );

    assert_eq!(
        fonto_schema.attributes().len(),
        schema.types_attribute().len()
    );

    let taskdef = schema.get_group_by_name("Task").unwrap();
    assert_eq!(1, taskdef.attributes().as_vec().len());

    // 3 attribute: @assigned?: AssignedPersonIds
    // @id: string
    assert_eq!(2, schema.types_attribute().len());
    assert_eq!(2, fonto_schema.attributes().len());

    // todo: other validation
}
