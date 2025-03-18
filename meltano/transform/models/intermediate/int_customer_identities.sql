select
    customer_id,
    first_name,
    last_name,
    date_of_birth,
    gender,
    countries.code as country_of_residence_code,
    nationalities.code as nationality_code,
{% if target.type == 'bigquery' %}
    questionnaires[safe_offset(0)].occupation_code as occupation_code,
    questionnaires[safe_offset(0)].economic_activity_code as economic_activity_code,
    questionnaires[safe_offset(0)].tax_id_number as tax_id_number,
    id_documents[safe_offset(0)].number as passport_number
{% elif target.type == 'snowflake' %}
    TO_VARCHAR(GET(questionnaires, 0):occupation_code) as occupation_code,
    TO_VARCHAR(GET(questionnaires, 0):economic_activity_code) as economic_activity_code,
    TO_VARCHAR(GET(questionnaires, 0):tax_id_number) as tax_id_number,
    TO_VARCHAR(GET(id_documents, 0):number) as passport_number
{% endif %}

from {{ ref('int_sumsub_applicants') }}
left join {{ ref('static_npb4_17_31_codigos_de_paises_o_territorios') }} as countries
    on
        countries.iso_alpha_3_code
    {% if target.type == 'bigquery' %}
        = questionnaires[safe_offset(0)].country_of_residence_iso_alpha_3_code
    {% elif target.type == 'snowflake' %}
        = TO_VARCHAR(GET(questionnaires, 0):country_of_residence_iso_alpha_3_code)
    {% endif %}
left join {{ ref('static_npb4_17_31_codigos_de_paises_o_territorios') }} as nationalities
    on nationalities.iso_alpha_3_code = nationality_iso_alpha_3_code
