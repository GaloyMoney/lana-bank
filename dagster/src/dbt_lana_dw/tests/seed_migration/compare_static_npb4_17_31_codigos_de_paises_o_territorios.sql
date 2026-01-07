-- Compare legacy SQL model with new seed implementation
-- This test should return 0 rows if both are equivalent

(
    select code, nationality, country, iso_alpha_3_code
    from {{ ref('static_npb4_17_31_codigos_de_paises_o_territorios_legacy') }}
    except distinct
    select code, nationality, country, iso_alpha_3_code
    from {{ ref('static_npb4_17_31_codigos_de_paises_o_territorios') }}
)
union all
(
    select code, nationality, country, iso_alpha_3_code
    from {{ ref('static_npb4_17_31_codigos_de_paises_o_territorios') }}
    except distinct
    select code, nationality, country, iso_alpha_3_code
    from {{ ref('static_npb4_17_31_codigos_de_paises_o_territorios_legacy') }}
)
