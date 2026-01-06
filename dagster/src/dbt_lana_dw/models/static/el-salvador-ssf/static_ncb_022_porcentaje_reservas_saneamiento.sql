select
    'A1' as category,
    0.00 as reserve_percentage,
    0 as consumer_calendar_ge_days,
    7 as consumer_calendar_le_days
union all
select 'A2', 0.01, 8, 30
union all
select 'B', 0.05, 31, 60
union all
select 'C1', 0.15, 61, 90
union all
select 'C2', 0.25, 91, 120
union all
select 'D1', 0.50, 121, 150
union all
select 'D2', 0.75, 151, 180
union all
select 'E', 1.00, 181, 50000
