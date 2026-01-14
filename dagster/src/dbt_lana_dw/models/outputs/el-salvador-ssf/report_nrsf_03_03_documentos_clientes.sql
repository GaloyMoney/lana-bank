select
    left(`NIU`, 25) as `NIU`,
    left(`Código del Documento`, 5) as `Código del Documento`,
    left(`Número de documento`, 25) as `Número de documento`
from
    {{ ref('int_nrsf_03_03_documentos_clientes') }}
