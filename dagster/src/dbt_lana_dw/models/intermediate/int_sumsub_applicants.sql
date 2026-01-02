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
    json_value(parsed_content.info.addresses[0].formattedAddress) as formatted_address,
    array(
        select as struct
            json_value(doc.country) as iso_alpha_3_code,
            json_value(doc.idDocType) as document_type,
            json_value(doc.number) as number
        from unnest(json_query_array(parsed_content.info.idDocs)) as doc
    ) as id_documents,

    array(
        select as struct
            json_value(questions.sections.personalInformation.items.occupation.value)as occupation_code,
            json_value(questions.sections.personalInformation.items.nit.value)as nit,
            json_value(questions.sections.personalInformation.items.sourceOfFunds.value)as source_of_funds,
            json_value(questions.sections.personalInformation.items.secondNationality.value)as second_nationality,
            json_value(questions.sections.personalInformation.items.maritalStatus.value)as marital_status,
            json_value(questions.sections.personalInformation.items.marriedName.value)as married_name,
            json_value(questions.sections.personalInformation.items.economicActivity.value)as economic_activity_code,
            json_value(questions.sections.personalInformation.items.countryOfResidence.value)as country_of_residence_iso_alpha_3_code,
            json_value(questions.sections.personalInformation.items.taxIdentificationNum.value) as tax_id_number,
            json_value(questions.sections.personalInformation.items.phoneNumber.value) as phone_number,
            json_value(questions.sections.personalInformation.items.relationshipToBank.value) as relationship_to_bank,
            json_value(questions.sections.personalInformation.items.DUI.value) as dui,
            json_value(questions.sections.personalInformation.items.municipality.value) as el_salvador_municipality
        from unnest(json_query_array(parsed_content.questionnaires)) as questions
    ) as questionnaires

from {{ ref('stg_sumsub_applicants') }}

where parsed_content is not null
	and parsed_content.errorCode is null
