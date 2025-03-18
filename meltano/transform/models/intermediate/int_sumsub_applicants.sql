{% if target.type == 'bigquery' %}
select
    customer_id,
    json_value(parsed_content.id) as applicant_id,
    timestamp(json_value(parsed_content.createdAt)) as created_at,
    json_value(parsed_content.info.firstName) as first_name,
    json_value(parsed_content.info.lastName) as last_name,
    date(json_value(parsed_content.info.dob)) as date_of_birth,
    json_value(parsed_content.info.gender) as gender,
    json_value(parsed_content.info.country) as iso_alpha_3_code,
    json_value(parsed_content.info.nationality) as nationality_iso_alpha_3_code,
    array(
        select as struct
            json_value(doc.country) as iso_alpha_3_code,
            json_value(doc.idDocType) as document_type,
            json_value(doc.number) as number
        from unnest(json_query_array(parsed_content.info.idDocs)) as doc
    ) as id_documents,

    array(
        select as struct
            json_value(questions.sections.personalInformation.items.occupation.value)
                as occupation_code,
            json_value(questions.sections.personalInformation.items.economicActivity.value)
                as economic_activity_code,
            json_value(questions.sections.personalInformation.items.countryOfResidence.value)
                as country_of_residence_iso_alpha_3_code,
            json_value(
                questions.sections.personalInformation.items.taxIdentificationNum.value
            ) as tax_id_number
        from unnest(json_query_array(parsed_content.questionnaires)) as questions
    ) as questionnaires
from {{ ref('stg_sumsub_applicants') }}
where parsed_content is not null
	and parsed_content.errorCode is null
{% elif target.type == 'snowflake' %}
with applicants as(
    select
        customer_id,
        TO_VARCHAR(parsed_content:id) as applicant_id,
        TO_TIMESTAMP(TO_VARCHAR(parsed_content:createdAt)) as created_at,
        TO_VARCHAR(parsed_content:info.firstName) as first_name,
        TO_VARCHAR(parsed_content:info.lastName) as last_name,
        date(TO_VARCHAR(parsed_content:info.dob)) as date_of_birth,
        TO_VARCHAR(parsed_content:info.gender) as gender,
        TO_VARCHAR(parsed_content:info.country) as iso_alpha_3_code,
        TO_VARCHAR(parsed_content:info.nationality) as nationality_iso_alpha_3_code,
        parsed_content:info.idDocs as docs,
        parsed_content:questionnaires as questions,
    from {{ ref('stg_sumsub_applicants') }}
    where parsed_content is not null
	and JSON_EXTRACT_PATH_TEXT(parsed_content, 'errorCode') is null
)
, id_docs as (
    select
    customer_id,
    array_agg(
        OBJECT_CONSTRUCT(
            'iso_alpha_3_code2', flat_docs.value:country,
            'document_type', flat_docs.value:idDocType,
            'number', flat_docs.value:number
    )) as id_documents
    from applicants a,
    table(flatten(input=>a.docs)) as flat_docs
    group by customer_id
)
, questions as (
    select
    customer_id,
    array_agg(
        OBJECT_CONSTRUCT(
            'occupation_code', flat_questions.value:sections.personalInformation.items.occupation.value,
            'economic_activity_code', flat_questions.value:sections.personalInformation.items.economicActivity.value,
            'country_of_residence_iso_alpha_3_code', flat_questions.value:sections.personalInformation.items.countryOfResidence.value,
            'tax_id_number', flat_questions.value:sections.personalInformation.items.taxIdentificationNum.value
    )) as questionnaires
    from applicants a,
    table(flatten(input=>questions)) as flat_questions
    group by customer_id
)

select
    a.* exclude (docs, questions),
    id_docs.* exclude (customer_id),
    questions.* exclude (customer_id)
from applicants a
left join id_docs using(customer_id)
left join questions using(customer_id)
{% endif %}
