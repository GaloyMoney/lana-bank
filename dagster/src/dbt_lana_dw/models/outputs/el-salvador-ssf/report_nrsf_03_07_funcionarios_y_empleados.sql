select
    left(`Primer Nombre`, 30) as `Primer Nombre`,
    left(`Segundo Nombre`, 30) as `Segundo Nombre`,
    left(`Primer Apellido`, 30) as `Primer Apellido`,
    left(`Segundo Apellido`, 30) as `Segundo Apellido`,
    left(`Apellido de casada`, 30) as `Apellido de casada`,
    format_date('%Y%m%d', cast(`Fecha de ingreso` as date)) as `Fecha de ingreso`,
    left(`Cargo`, 50) as `Cargo`,
    left(`NIU`, 25) as `NIU`,
    left(`Código del documento`, 5) as `Código del documento`,
    left(`Número de documento`, 25) as `Número de documento`,
    left(`Número Telefónico`, 10) as `Número Telefónico`,
    left(`Departamento`, 25) as `Departamento`,
    left(`Relacionado por administración`, 1) as `Relacionado por administración`
from {{ ref("int_nrsf_03_07_funcionarios_y_empleados") }}
