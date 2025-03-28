select
    customer_id,
    first_name,
    last_name,
    date_of_birth,
    gender,
    countries.code as country_of_residence_code,
    countries.iso_alpha_3_code as country_of_residence_alpha_3_code,
    nationalities.code as nationality_code,
    formatted_address,
    questionnaires[safe_offset(0)].occupation_code as occupation_code,
    questionnaires[safe_offset(0)].economic_activity_code as economic_activity_code,
    questionnaires[safe_offset(0)].tax_id_number as tax_id_number,
    questionnaires[safe_offset(0)].phone_number as phone_number,
    questionnaires[safe_offset(0)].relationship_to_bank as relationship_to_bank,
    questionnaires[safe_offset(0)].dui as dui,
    questionnaires[safe_offset(0)].el_salvador_municipality as el_salvador_municipality,
    questionnaires[safe_offset(0)].marital_status as marital_status,
    id_documents[safe_offset(0)].number as passport_number

from {{ ref('int_sumsub_applicants') }}
left join {{ ref('static_npb4_17_31_codigos_de_paises_o_territorios') }} as countries
    on
        countries.iso_alpha_3_code
        = questionnaires[safe_offset(0)].country_of_residence_iso_alpha_3_code
left join {{ ref('static_npb4_17_31_codigos_de_paises_o_territorios') }} as nationalities
    on nationalities.iso_alpha_3_code = nationality_iso_alpha_3_code
