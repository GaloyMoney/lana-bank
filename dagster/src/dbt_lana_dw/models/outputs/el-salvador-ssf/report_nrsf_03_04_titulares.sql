select left(`NIU`, 25) as `NIU`, left(`Número de cuenta`, 20) as `Número de cuenta`
from {{ ref("int_nrsf_03_04_titulares") }}
