{% macro create_udf_json_array_to_code() %}

CREATE OR REPLACE FUNCTION {{target.schema}}.udf_json_array_to_code (json_array_of_code_obj JSON, separator STRING)
RETURNS STRING
LANGUAGE js
AS r"""
    var codes = [];
    var parsedJSON = JSON.parse(json_array_of_code_obj);
    for (var i=0;i<parsedJSON.length;i++) {
        codes.push(parsedJSON[i].code);
    }
    return codes.join(separator)
"""

{% endmacro %}
