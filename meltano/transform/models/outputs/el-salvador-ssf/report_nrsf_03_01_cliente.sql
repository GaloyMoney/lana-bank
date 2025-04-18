select
    cast(round(`Saldo garantizado`, 2) as string) as `Saldo garantizado`,
    left(`NIU`, 25) as `NIU`,
    left(`Primer Nombre`, 30) as `Primer Nombre`,
    left(`Segundo Nombre`, 30) as `Segundo Nombre`,
    left(`Tercer Nombre`, 30) as `Tercer Nombre`,
    left(`Primer Apellido`, 30) as `Primer Apellido`,
    left(`Segundo Apellido`, 30) as `Segundo Apellido`,
    left(`Apellido de casada`, 30) as `Apellido de casada`,
    left(`Razón social`, 80) as `Razón social`,
    left(`Tipo de persona`, 1) as `Tipo de persona`,
    left(`Nacionalidad`, 4) as `Nacionalidad`,
    left(`Actividad Económica`, 6) as `Actividad Económica`,
    left(`País de Residencia`, 6) as `País de Residencia`,
    left(`Departamento`, 2) as `Departamento`,
    left(`Distrito`, 2) as `Distrito`,
    left(`Dirección`, 100) as `Dirección`,
    left(`Número de teléfono fijo`, 30) as `Número de teléfono fijo`,
    left(`Número de celular`, 30) as `Número de celular`,
    left(`Correo electrónico`, 50) as `Correo electrónico`,
    left(`Es residente`, 1) as `Es residente`,
    left(`Tipo de sector`, 1) as `Tipo de sector`,
    format_date('%Y%m%d', cast(`Fecha de Nacimiento` as date)) as `Fecha de Nacimiento`,
    left(`Género`, 1) as `Género`,
    left(`Estado civil`, 1) as `Estado civil`,
    left(`Clasificación de Riesgo`, 2) as `Clasificación de Riesgo`,
    left(`Tipo de relación`, 1) as `Tipo de relación`,
    left(`Agencia`, 7) as `Agencia`
from
    {{ ref('int_nrsf_03_01_cliente') }}
