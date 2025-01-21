WITH set_hierarchy_strings AS (

	SELECT expanded.account_set_id, expanded.member_id, expanded.member_type,
		STRING_AGG(set_name, ":" ORDER BY o) AS set_hierarchy_string

	FROM {{ ref('int_account_sets_expanded') }} expanded,
		UNNEST(set_hierarchy) parent_set_id WITH OFFSET o

	JOIN {{ ref('int_account_sets') }} sets ON parent_set_id = sets.account_set_id

	WHERE set_name != "Balance Sheet"

	GROUP BY account_set_id, member_id, member_type

)

SELECT
	member_id AS id_codigo_cuenta,
	set_hierarchy_string || ":" || account_name AS nom_cuenta,
	COALESCE(CASE
		WHEN normal_balance_type = "credit" THEN settled_cr - settled_dr
		WHEN normal_balance_type = "debit" THEN settled_dr - settled_cr
	END, 0) AS valor,

FROM set_hierarchy_strings

LEFT JOIN {{ ref('int_account_sets') }} USING (account_set_id)

LEFT JOIN {{ ref('int_accounts') }} accounts
	ON accounts.account_id = member_id

LEFT JOIN {{ ref('int_account_balances') }} balances
	ON balances.account_id = member_id

WHERE member_type = "Account"
	AND set_name = "Balance Sheet"
