{{ config(materialized='table') }}

select
    customer_id,

    first_name,
    last_name,
    date_of_birth,
    gender,
    iso_alpha_3_code,
    nationality_iso_alpha_3_code,
    formatted_address,
    questionnaires[safe_offset(0)].occupation_code as occupation_code,
    questionnaires[safe_offset(0)].economic_activity_code as economic_activity_code,
    questionnaires[safe_offset(0)].tax_id_number as tax_id_number,
    questionnaires[safe_offset(0)].phone_number as phone_number,
    questionnaires[safe_offset(0)].relationship_to_bank as relationship_to_bank,
    questionnaires[safe_offset(0)].dui as dui,
    questionnaires[safe_offset(0)].el_salvador_municipality as el_salvador_municipality,
    questionnaires[safe_offset(0)].marital_status as marital_status,
    questionnaires[safe_offset(0)].married_name as married_name,
    questionnaires[safe_offset(0)].nit as nit,
    questionnaires[safe_offset(0)].source_of_funds as source_of_funds,
    questionnaires[safe_offset(0)].second_nationality as second_nationality,
    id_documents[safe_offset(0)].number as passport_number

from {{ ref('int_core_customer_events_rollup') }}
inner join {{ ref('int_sumsub_applicants') }} using (customer_id)
