---
id: terms
title: Términos
sidebar_position: 6
---

# Términos

Términos es un **objeto de valor** que captura los parámetros bajo los cuales opera una facilidad de crédito.
Se copia en una facilidad cuando la facilidad es creada y no cambia después.

## Campos

La estructura `ValoresDeTérminos` contiene los siguientes campos:

- `tasa_anual` – tasa de interés cobrada sobre el principal pendiente.
- `duración` – longitud total de la facilidad.
- `duración_vencimiento_intereses_desde_devengo` – tiempo desde el devengo de intereses hasta cuando ese interés vence.
- `duración_obligación_vencida_desde_vencimiento` – período de gracia opcional antes de que una obligación vencida se considere en mora.
- `duración_liquidación_obligación_desde_vencimiento` – buffer opcional antes de que una obligación en mora sea elegible para liquidación.
- `intervalo_ciclo_devengo` – cadencia con la que se generan nuevas obligaciones de interés.
- `intervalo_devengo` – frecuencia utilizada para calcular el interés devengado dentro de un ciclo.
- `tasa_comisión_única` – porcentaje de comisión tomado en el desembolso.
- `cvl_liquidación` – límite de valor del colateral que activa la liquidación.
- `cvl_llamada_margen` – límite de valor del colateral que activa una llamada de margen.
- `cvl_inicial` – límite de valor del colateral requerido en la creación de la facilidad.

## Plantillas de Términos

`PlantillaDeTérminos` es una entidad utilizada para persistir un conjunto reutilizable de valores de términos.
Las facilidades de crédito **no** están vinculadas a plantillas; en su lugar, los valores de una plantilla son
copiados en la facilidad en el momento de la creación.
