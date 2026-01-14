select
    left(`Código de la Agencia`, 7) as `Código de la Agencia`,
    left(`Nombre de la Agencia`, 30) as `Nombre de la Agencia`,
    left(`Ubicación de la Agencia`, 100) as `Ubicación de la Agencia`,
    left(`Código del Departamento`, 2) as `Código del Departamento`,
    left(`Código del Distrito`, 2) as `Código del Distrito`,
    left(`Estado de la Agencia`, 1) as `Estado de la Agencia`,
    current_timestamp() as created_at
from
    {{ ref('int_nrsf_03_05_agencias') }}
