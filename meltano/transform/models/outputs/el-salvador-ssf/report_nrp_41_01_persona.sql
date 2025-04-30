{{ config(materialized='table') }}

select
    cast(`pais_residencia` as string) as `pais_residencia`,
    cast(`nacionalidad` as string) as `nacionalidad`,
    left(`nit_persona`, 14) as `nit_persona`,
    left(`dui`, 9) as `dui`,
    left(`primer_apellido`, 25) as `primer_apellido`,
    left(`segundo_apellido`, 25) as `segundo_apellido`,
    left(`apellido_casada`, 25) as `apellido_casada`,
    left(`primer_nombre`, 25) as `primer_nombre`,
    left(`segundo_nombre`, 25) as `segundo_nombre`,
    left(`nombre_sociedad`, 100) as `nombre_sociedad`,
    left(`tipo_persona`, 1) as `tipo_persona`,
    left(`tipo_relacion`, 1) as `tipo_relacion`,
    left(`tipo_identificador`, 1) as `tipo_identificador`,
    left(`nit_desactualizado`, 14) as `nit_desactualizado`,
    left(`residente`, 1) as `residente`,
    left(`giro_persona`, 6) as `giro_persona`,
    left(`tamano_empresa`, 2) as `tamano_empresa`,
    left(`tipo_empresa`, 1) as `tipo_empresa`,
    format('%.2f', round(`reserva`, 2)) as `reserva`,
    left(`categoria_riesgo`, 2) as `categoria_riesgo`,
    left(`numero_cliente`, 17) as `numero_cliente`,
    left(`id_alterno`, 20) as `id_alterno`,
    left(`tipo_id_alterno`, 2) as `tipo_id_alterno`,
    format_date('%Y-%m-%d', cast(`fecha_nacimiento` as date)) as `fecha_nacimiento`,
    format('%.2f', round(`riesgo_consolidado`, 2)) as `riesgo_consolidado`,
    left(`sexo_persona`, 1) as `sexo_persona`,
    left(`ocupación`, 3) as `ocupación`,
    left(`id_pais_origen`, 20) as `id_pais_origen`,
    left(`nit_anterior`, 14) as `nit_anterior`,
    left(`tipo_ident_anterior`, 1) as `tipo_ident_anterior`,
    left(`distrito_residencia`, 4) as `distrito_residencia`

from
    {{ ref('int_nrp_41_01_persona') }}
