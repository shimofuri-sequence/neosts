use sheet::CellValue;

const CHECKBOX_PLUGIN_ID: &str = "com.leftangle.AppLib.CheckBoxParam";
const TIME_REMAP_PLUGIN_ID: &str = "com.leftangle.AppLib.TimeParam";

pub fn keyframe_data(_column_name: &str, values: &[CellValue], fps: u32) -> String {
    let fps = fps.max(1) as f64;
    let has_karacel = values.iter().any(is_karacel_value);
    let mut output = String::new();

    output.push_str("// Layer-stack Selection Data\n");
    output.push_str("{\n");
    output.push_str("  \"Params\": {\n");

    if has_karacel {
        output.push_str("    \"animatedVisibility\": {\n");
        output.push_str(&format!(
            "      \"PluginID\": \"{}\",\n",
            CHECKBOX_PLUGIN_ID
        ));
        output.push_str("      \"curve\": [\n");

        let mut last_visible: Option<bool> = None;
        let mut first_entry = true;
        for (frame, value) in values.iter().enumerate() {
            let visible = !is_karacel_value(value);
            if last_visible == Some(visible) {
                continue;
            }
            if !first_entry {
                output.push_str(",\n");
            }
            output.push_str("        \"Constant\",\n");
            output.push_str(&format!("        {:.16},\n", frame as f64 / fps));
            output.push_str(&format!("        {}", visible));
            first_entry = false;
            last_visible = Some(visible);
        }

        output.push_str("\n      ]\n");
        output.push_str("    },\n");
    }

    output.push_str("    \"enableTimeRemap\": {\n");
    output.push_str(&format!(
        "      \"PluginID\": \"{}\",\n",
        CHECKBOX_PLUGIN_ID
    ));
    output.push_str("      \"value\": true\n");
    output.push_str("    },\n");

    output.push_str("    \"timeRemap\": {\n");
    output.push_str(&format!(
        "      \"PluginID\": \"{}\",\n",
        TIME_REMAP_PLUGIN_ID
    ));
    output.push_str("      \"curve\": [\n");

    let mut last_value: Option<i64> = None;
    let mut first_entry = true;
    output.push_str("        \"Constant\",\n");
    for (frame, value) in values.iter().enumerate() {
        let value = numeric_cell_value(value);
        if last_value == Some(value) {
            continue;
        }
        if !first_entry {
            output.push_str(",\n");
        }

        let remap_time = if value > 0 {
            (value - 1) as f64 / fps
        } else {
            0.0
        };

        output.push_str(&format!("        {:.16},\n", frame as f64 / fps));
        output.push_str(&format!("        {:.16}", remap_time));
        first_entry = false;
        last_value = Some(value);
    }

    output.push_str("\n      ]\n");
    output.push_str("    }\n");
    output.push_str("  }\n");
    output.push('}');
    output
}

fn numeric_cell_value(value: &CellValue) -> i64 {
    value.as_i64().max(0)
}

fn is_karacel_value(value: &CellValue) -> bool {
    value.as_i64() == 0
}
