select
    customer_id,
    json_value(parsed_content.id) as applicant_id,
    timestamp(json_value(parsed_content.createdat)) as created_at,
    json_value(parsed_content.info.firstname) as first_name,
    json_value(parsed_content.info.lastname) as last_name,
    date(json_value(parsed_content.info.dob)) as date_of_birth,
    json_value(parsed_content.info.gender) as gender,
    json_value(parsed_content.info.country) as iso_alpha_3_code,
    json_value(parsed_content.info.nationality) as nationality_iso_alpha_3_code,
    array(
        select as struct
            json_value(doc.country) as iso_alpha_3_code,
            json_value(doc.iddoctype) as document_type,
            json_value(doc.number) as number
        from unnest(json_query_array(parsed_content.info.iddocs)) as doc
    ) as id_documents,

    array(
        select as struct
            json_value(questions.sections.personalinformation.items.occupation.value)
                as occupation_code,
            json_value(questions.sections.personalinformation.items.economicactivity.value)
                as economic_activity_code,
            json_value(questions.sections.personalinformation.items.countryofresidence.value)
                as country_of_residence_iso_alpha_3_code,
            json_value(
                questions.sections.personalinformation.items.taxidentificationnum.value
            ) as tax_id_number
        from unnest(json_query_array(parsed_content.questionnaires)) as questions
    ) as questionnaires

from {{ ref('stg_sumsub_applicants') }}

where
    parsed_content is not null
    and parsed_content.errorcode is null
