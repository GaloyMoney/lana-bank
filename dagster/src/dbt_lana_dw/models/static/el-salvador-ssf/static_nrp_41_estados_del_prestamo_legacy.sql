-- Estado               Explicación
--
-- Vigente - al día     (Préstamo sin retrasos)
-- Vigente - en mora    (Préstamo con mora hasta 90 días)
-- Vencido              (Préstamo con mora de 91 días o más)
-- Cancelado            (Préstamo liquidado o cancelado)
-- En cobro judicial    (Préstamo con marca manual de estado en cobro judicial.)
-- Saneado              (Préstamo irrecuperable que ha sido liquidado con fondos
-- propios)
--
-- Status               Explanation
--
-- Current - current    (Loan with no delays)
-- Current - past due   (Loan up to 90 days past due)
-- Passed               (Loan 91 days or more past due)
-- Paid off             (Loan settled or canceled)
-- In collection        (Loan with manual status in collection)
-- Written off             (Irrecoverable loan that has been settled with own funds)
select
    'Vigente - al día' as estado,
    'Préstamo sin retrasos' as `explicación`,
    'Current - current' as status,
    'Loan with no delays' as explanation,
    -50000 as consumer_calendar_ge_days,
    0 as consumer_calendar_le_days
union all
select
    'Vigente - en mora',
    'Préstamo con mora hasta 90 días',
    'Current - past due',
    'Loan up to 90 days past due',
    1,
    90
union all
select
    'Vencido',
    'Préstamo con mora de 91 días o más',
    'Passed',
    'Loan 91 days or more past due',
    91,
    50000
