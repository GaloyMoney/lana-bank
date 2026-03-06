# NRSF-03 Seeding Scenarios Proposal

## Overview

This document proposes data seeding scenarios for **NRSF-03** — the El Salvador regulation titled _"Normas Técnicas para la Generación de Información de los Depósitos Monetarios y sus Titulares"_. The goal is to produce realistic data in Lana's Postgres event-sourced tables that, once processed through the dbt pipeline and report generation, yields complete and correct regulatory files.

NRSF-03 mandates **9 report files** (Archivos 01–09) covering clients, deposits, client documents, account holders, branches, products, employees, guaranteed deposit summaries, and adjustments. Each report has specific field requirements detailed in Annexo No. 1 of the norm.

---

## Dependency Chain Summary

```
Report files (TXT/CSV, pipe-delimited)
  └── dagster/generate_es_reports (Python: fetches BigQuery tables, formats output)
       └── dbt output models (report_nrsf_03_*)  — field width truncation, date formatting
            └── dbt intermediate models (int_nrsf_03_*)  — business logic transformation
                 ├── int_core_customer_events_rollup  ← source: core_customer_events_rollup
                 ├── int_customer_identities           ← source: sumsub_applicants_dlt (Sumsub)
                 ├── int_deposit_balances               ← sources: core_deposit_events_rollup,
                 │                                                  core_withdrawal_events_rollup
                 ├── int_core_deposit_account_events_rollup ← source: core_deposit_account_events_rollup
                 ├── int_approved_credit_facilities     ← sources: core_credit_facility_events_rollup,
                 │                                                  core_disbursal_events_rollup,
                 │                                                  core_interest_accrual_cycle_events_rollup,
                 │                                                  core_payment_events_rollup, etc.
                 ├── stg_core_public_ids                ← source: core_public_ids
                 └── stg_bitfinex_ticker_price          ← source: bitfinex_ticker_dlt
```

### Lana Source Tables Required

| Postgres Table | Lana Entity | Key Events |
|---|---|---|
| `core_customer_events_rollup` | Customer | `Initialized`, `PersonalInfoUpdated`, `ActivityUpdated` |
| `core_deposit_account_events_rollup` | DepositAccount | `Initialized`, `AccountHolderStatusUpdated`, `Frozen`, `Closed` |
| `core_deposit_events_rollup` | Deposit | `Initialized` (confirmed deposits) |
| `core_withdrawal_events_rollup` | Withdrawal | `Initialized`, `Confirmed` (approved withdrawals) |
| `core_credit_facility_events_rollup` | CreditFacility | `Initialized`, `CollateralizationStateChanged` |
| `core_disbursal_events_rollup` | Disbursal | `Initialized`, settled disbursals |
| `core_interest_accrual_cycle_events_rollup` | InterestAccrualCycle | Accrual events |
| `core_payment_events_rollup` | Payment | Payment recordings |
| `core_payment_allocation_events_rollup` | PaymentAllocation | Interest vs principal splits |
| `core_pending_credit_facility_events_rollup` | PendingCreditFacility | Proposal approval flow |
| `core_credit_facility_proposal_events_rollup` | CreditFacilityProposal | Terms and approval |
| `core_public_ids` | PublicId | ID assignments for customers, accounts, facilities |

### External Data Sources Required

| Source | Purpose |
|---|---|
| `sumsub_applicants_dlt` | KYC identity data: names, DUI, NIT, passport, address, nationality, etc. |
| `bitfinex_ticker_dlt` | BTC/USD price for deposit balance conversion |

---

## Report-by-Report Analysis and Required Scenarios

### Report 01 — CLIENTE (Clients)

**Norm requirement (Art. 4, Annexo 1 §1):** All depositors/clients of the entity. 27 fields including NIU, full name, social reason, person type, nationality, economic activity, country of residence, department, district, address, phone, email, residency status, sector type, DOB, gender, marital status, risk classification, relationship type, branch, and guaranteed balance.

**dbt model:** `int_nrsf_03_01_cliente` reads from:
- `int_core_customer_events_rollup` → customer_id, customer_type, email
- `int_customer_identities` → first_name, last_name, date_of_birth, gender, nationality, address, phone_number, economic_activity_code, tax_id_number, dui, marital_status, married_name, relationship_to_bank, country_of_residence_code
- `int_approved_credit_facilities` → total_collateral_amount_usd (for "Saldo garantizado", capped at $10,289)
- `stg_core_public_ids` → public ID as NIU

**Current gaps:** Hardcoded department=15, district=00, es_residente=0, tipo_sector=1. No branch (Agencia) assignment.

#### Scenarios for Report 01

**Scenario 1.1 — Natural person, resident, with active credit facility**
- **Actions in Lana:**
  1. Create a customer via KYC onboarding (triggers `CustomerEvent::Initialized`)
  2. Complete Sumsub KYC with full identity data: first name, last name, DOB, gender=M, nationality=SLV, address in San Salvador, phone, email, NIT, DUI, marital_status=single, economic_activity_code, country_of_residence=SLV
  3. Create and approve a credit facility with BTC collateral (triggers `CreditFacilityEvent::Initialized` with collateral_amount)
  4. Ensure collateral USD value > $10,289 to test the `deposits_coverage_limit` cap
- **Expected events:** `CustomerEvent::Initialized`, `CreditFacilityEvent::Initialized`, `CollateralizationStateChanged`
- **Norm justification:** Art. 10 requires 100% of client data. Fields #1-27 of Archivo de Clientes. Tests the guaranteed balance cap per Art. 167 of Ley de Bancos and NRSF-01 Art. 6.

**Scenario 1.2 — Natural person, non-resident, without credit facility**
- **Actions in Lana:**
  1. Create customer with nationality from a non-SLV country (e.g., USA)
  2. Sumsub KYC with country_of_residence != SLV, passport (no DUI)
  3. No credit facility created
- **Expected events:** `CustomerEvent::Initialized`
- **Norm justification:** Field #19 "Es residente" must be 0 for non-residents. Field #27 "Saldo garantizado" should be 0.00 or NULL since there's no collateral. Tests that clients without active operations get "NA" risk classification (field #24).

**Scenario 1.3 — Natural person, female, married, with married name**
- **Actions in Lana:**
  1. Create customer with gender=F
  2. Sumsub KYC includes married_name, marital_status=2 (casada)
- **Expected events:** `CustomerEvent::Initialized`
- **Norm justification:** Tests field #7 "Apellido de casada" which is specific to married women. Also validates gender encoding (F) and marital status encoding (2).

**Scenario 1.4 — Multiple customers with varying collateral levels**
- **Actions in Lana:**
  1. Customer A: collateral worth $15,000 (above the $10,289 limit)
  2. Customer B: collateral worth $5,000 (below limit)
  3. Customer C: collateral worth exactly $10,289
- **Expected events:** Multiple `CreditFacilityEvent::Initialized` and `CollateralizationStateChanged`
- **Norm justification:** Validates the guaranteed deposit limit calculation (fields #27). The limit of $10,289 is per person per Art. 167 of Ley de Bancos.

---

### Report 02 — DEPOSITOS (Deposits)

**Norm requirement (Art. 4, Annexo 1 §2):** Deposit detail by product type and client accounts. 34 fields including product code, account number, branch, interest periodicity, rates, dates, special conditions, balances (capital, interest, total), status, currency, etc.

**dbt model:** `int_nrsf_03_02_depositos` reads from:
- `int_deposit_balances` → deposit_account_balance_usd, earliest_recorded_at, latest_recorded_at
- `int_core_deposit_account_events_rollup` → deposit_account_id, customer_id, status
- `int_core_customer_events_rollup` → customer data
- `int_approved_credit_facilities` → total_collateral_amount_usd
- `stg_bitfinex_ticker_price` → BTC/USD price for "Saldo de capital" calculation

**Current state:** Product code hardcoded as "BTCL", many fields use placeholder values. Balance computation: deposits minus approved withdrawals. "Saldo de capital" converts BTC sats to USD via latest Bitfinex price.

#### Scenarios for Report 02

**Scenario 2.1 — Active deposit account with deposits and no withdrawals**
- **Actions in Lana:**
  1. Create customer + complete KYC
  2. Create deposit account (triggers `DepositAccountEvent::Initialized`)
  3. Record 3 deposits of varying amounts (triggers `DepositEvent::Initialized` × 3)
  4. Ensure BTC price data exists in Bitfinex feed
- **Expected events:** `DepositAccountEvent::Initialized`, `DepositEvent::Initialized` × 3
- **Norm justification:** Tests all 34 fields of Archivo de Depósitos. Validates saldo_total = sum of deposits. Tests "Fecha de apertura" (field #20) = earliest deposit date.

**Scenario 2.2 — Deposit account with deposits and partial withdrawal**
- **Actions in Lana:**
  1. Create deposit account with initial deposit of $10,000 equivalent in sats
  2. Record an approved withdrawal of $3,000 equivalent
  3. Ensure Bitfinex BTC/USD price is available
- **Expected events:** `DepositEvent::Initialized`, `WithdrawalEvent::Initialized` + `Confirmed`
- **Norm justification:** Validates that "Saldo total" (field #33) = deposits - withdrawals. Tests "Fecha de la última transacción" (field #30) reflects the withdrawal date.

**Scenario 2.3 — Deposit account with collateral (linked to credit facility)**
- **Actions in Lana:**
  1. Customer has both deposit account and approved credit facility
  2. Deposit account has balance
  3. Credit facility has collateral in BTC
- **Expected events:** All deposit + credit facility events
- **Norm justification:** Tests "Fondos restringidos" (field #25) — deposits used as collateral should be reported. Also validates "Monto mínimo" (field #22) which maps to total_collateral_amount_usd.

**Scenario 2.4 — Multiple deposit accounts for the same customer**
- **Actions in Lana:**
  1. One customer with 2 deposit accounts
  2. Each with different deposit amounts and dates
- **Expected events:** `DepositAccountEvent::Initialized` × 2, multiple `DepositEvent::Initialized`
- **Norm justification:** Tests that "Número de cuenta" (field #2) is unique per deposit. Validates the relationship between Depósitos and Titulares files via account number.

**Scenario 2.5 — Deposit account with zero balance (all withdrawn)**
- **Actions in Lana:**
  1. Deposit $5,000 equivalent, then withdraw all
- **Expected events:** Deposit + full withdrawal events
- **Norm justification:** Tests edge case where "Saldo total" = 0. Account should still appear with status "Activa" (field #34) unless explicitly closed. Art. 15 validation requires no overdrafts.

---

### Report 03 — DOCUMENTOS CLIENTES (Client Documents)

**Norm requirement (Art. 4, Annexo 1 §3):** All current identification documents for each client. 3 fields: NIU, document code (NIT/DUI/PASAP/LICEC/PTNAC/CRESI/CMINO), document number.

**dbt model:** `int_nrsf_03_03_documentos_clientes` unpivots three document types from `int_customer_identities`:
- NIT (tax_id_number)
- DUI (dui)
- PASAP (passport_number)

#### Scenarios for Report 03

**Scenario 3.1 — Customer with all three document types**
- **Actions in Lana:**
  1. Create customer where Sumsub KYC provides NIT, DUI, and passport
- **Expected events:** `CustomerEvent::Initialized`
- **Norm justification:** Tests that one customer produces 3 rows in the documents file. Validates field #2 codes: NIT, DUI, PASAP.

**Scenario 3.2 — Customer with only passport (foreign national)**
- **Actions in Lana:**
  1. Create non-Salvadoran customer with passport only (no DUI, no NIT)
- **Expected events:** `CustomerEvent::Initialized`
- **Norm justification:** Tests that optional documents are correctly excluded. Only one PASAP row should appear.

**Scenario 3.3 — Customer with NIT and DUI (typical Salvadoran)**
- **Actions in Lana:**
  1. Create Salvadoran customer with NIT and DUI, no passport
- **Expected events:** `CustomerEvent::Initialized`
- **Norm justification:** Most common case for resident clients. Two rows per customer.

---

### Report 04 — TITULARES (Account Holders)

**Norm requirement (Art. 4, Annexo 1 §4):** Maps depositors to their accounts. 2 fields: NIU and account number (Número de cuenta). Must match with Clientes (via NIU) and Depósitos (via account number).

**dbt model:** `int_nrsf_03_04_titulares` joins deposit_balances → deposit_accounts → customers and maps public IDs for both.

#### Scenarios for Report 04

**Scenario 4.1 — Single holder, single account**
- **Actions in Lana:**
  1. One customer, one deposit account with at least one deposit
- **Expected events:** Customer + DepositAccount + Deposit events
- **Norm justification:** Tests basic NIU → account number mapping. Tipo de titularidad in Depósitos (field #15) = "1" (Único).

**Scenario 4.2 — Single holder, multiple accounts**
- **Actions in Lana:**
  1. One customer with 2+ deposit accounts, each with deposits
- **Expected events:** Multiple DepositAccount and Deposit events
- **Norm justification:** Tests that one NIU maps to multiple account numbers. Validates Número de titulares (field #16) count in Depósitos file.

*Note: Lana currently supports single-holder accounts only (tipo_titularidad hardcoded to "1"). Co-ownership (mancomunadas) scenarios are not yet supported.*

---

### Report 05 — AGENCIAS (Branches)

**Norm requirement (Art. 4, Annexo 1 §5):** Branch catalog. 6 fields: branch code, name, location, department code, district code, status.

**dbt model:** `int_nrsf_03_05_agencias` — **currently disabled** (`WHERE 1 = 0`). All fields are hardcoded as 'TODO'.

#### Scenarios for Report 05

**Scenario 5.1 — Single digital-only branch**
- **Actions in Lana:**
  - No specific Lana events needed. This is static/seed data.
  - Populate a reference table or configure the dbt model with the institution's single virtual branch.
- **Norm justification:** Art. 4, file #5 requires at least one agency record. As a digital bank, one virtual branch should suffice. Department=06 (San Salvador), District=14 (San Salvador district), Status=1 (Habilitado).

*Note: This report needs dbt model fixes — the `1 = 0` filter must be removed and real data populated. This may require a new source table or seed file.*

---

### Report 06 — PRODUCTOS (Products)

**Norm requirement (Art. 4, Annexo 1 §6):** Product catalog. 4 fields: product code, name, status (available/closed), generic product code (01=ahorro, 02=corriente, 03=plazo, 04=otros).

**dbt model:** `int_nrsf_03_06_productos` — static single row: BTCL / "Bitcoin Loan Collateral" / status=1 / generic_code=04 (Otros).

#### Scenarios for Report 06

**Scenario 6.1 — Single product (BTCL)**
- **Actions in Lana:**
  - No specific Lana events needed. Static data.
  - The single product "BTCL" mapped to generic code "04" (Otros) is appropriate since BTC collateral deposits don't fit traditional categories (ahorro/corriente/plazo).
- **Norm justification:** Art. 4, file #6. Product codes used in Depósitos (field #1) must exist in the Productos table. Currently hardcoded to BTCL.

*Note: If Lana adds new deposit product types in the future, this model must be extended.*

---

### Report 07 — FUNCIONARIOS Y EMPLEADOS (Officials and Employees)

**Norm requirement (Art. 4, Annexo 1 §7):** Employee and officer data. 13 fields: name, surname, married name, hire date, role, NIU, document code/number, phone, department, related-by-administration flag.

**dbt model:** `int_nrsf_03_07_funcionarios_y_empleados` — **currently disabled** (`WHERE customer_type = 'BankEmployee' AND 1 = 0`).

#### Scenarios for Report 07

**Scenario 7.1 — Bank employees registered as customers**
- **Actions in Lana:**
  1. Create customer entities with `customer_type = BankEmployee` (if this type is supported in production)
  2. Complete KYC with full personal data + employment info
- **Expected events:** `CustomerEvent::Initialized` with customer_type = BankEmployee
- **Norm justification:** Art. 4, file #7. Requires employee data linked via NIU. Field #13 "Relacionado por administración" per Art. 204 of Ley de Bancos.

*Note: This report needs model fixes — the `1 = 0` filter must be removed. The current model uses customer entities for employee data, which may need an alternative approach if employees aren't modeled as customers.*

---

### Report 08 — RESUMEN DE DEPOSITOS GARANTIZADOS (Guaranteed Deposits Summary)

**Norm requirement (Art. 4, Annexo 1 §8):** Per-depositor summary for IGD (Instituto de Garantía de Depósitos) certification. 15 fields: correlative, NIU, full name, married name, social reason, document code/number, total accounts, capital balance, interest balance, guaranteed balance.

**dbt model:** `int_nrsf_03_08_resumen_de_depositos_garantizados` — **currently disabled** (`WHERE customer_type = 'NoType' AND 1 = 0`).

#### Scenarios for Report 08

**Scenario 8.1 — Guaranteed deposit summary with multiple depositors**
- **Actions in Lana:**
  1. Create 3+ customers, each with deposit accounts and varying balances
  2. Customer A: total balance $15,000 → guaranteed = $10,289 (capped)
  3. Customer B: total balance $5,000 → guaranteed = $5,000
  4. Customer C: total balance $0 → guaranteed = $0
- **Expected events:** All customer + deposit + withdrawal events for each customer
- **Norm justification:** Art. 173 of Ley de Bancos (IGD certification). "Saldo garantizado" (field #15) capped at $10,289 per person. "Total de cuentas" (field #12) counts accounts per depositor.

*Note: This report needs model fixes — the `1 = 0` filter must be removed. The model should aggregate deposit data per customer, compute total accounts, capital balance (from BTC→USD conversion), and cap guaranteed balance.*

---

### Report 09 — AJUSTES (Adjustments)

**Norm requirement (Art. 4, Annexo 1 §9):** Reconciliation adjustments. 3 fields: account number, adjustment amount, adjustment detail. Related to the COES (Sistema Contable Estadístico) validation per Art. 15.

**dbt model:** `int_nrsf_03_09_ajustes` — **currently disabled** (`WHERE customer_type = 'NoType' AND 1 = 0`).

#### Scenarios for Report 09

**Scenario 9.1 — No adjustments (clean month)**
- **Actions in Lana:**
  - Normal operations with no reconciliation discrepancies
- **Expected output:** Empty file (zero rows)
- **Norm justification:** Art. 15 — adjustments are only required when there are discrepancies between deposit files and the COES. An empty file is valid if the balance validation passes.

**Scenario 9.2 — Currency conversion adjustment**
- **Actions in Lana:**
  1. Deposits in BTC with fluctuating BTC/USD prices during the month
  2. End-of-month BTC/USD conversion creates a rounding discrepancy
- **Expected output:** Adjustment row with account number, amount = rounding difference, detail = "Ajuste por conversión de moneda BTC/USD"
- **Norm justification:** Art. 15 specifically mentions "ajustes por conversión de moneda". This is highly relevant for Lana since all deposits are in BTC but reporting is in USD.

---

## Comprehensive Seeding Plan

### Phase 1: Foundational Data (Prerequisites)

These must exist before any customer/deposit scenarios:

| Data | Source | Notes |
|---|---|---|
| BTC/USD price history | `bitfinex_ticker_dlt` | Need at least one recent price point. Seed ~$65,000 for realistic scenarios. |
| Country codes | `static_npb4_17_31_codigos_de_paises_o_territorios` | dbt seed, already exists |
| Public ID counter | `core_public_ids` | Auto-generated during entity creation |

### Phase 2: Customer Portfolio (5-7 Customers)

| Customer | Type | Nationality | Documents | Credit Facility | Purpose |
|---|---|---|---|---|---|
| **C1**: María López | Natural, Female, Married | SLV | NIT + DUI | Yes, collateral > $10,289 | Tests married name, DUI+NIT, guaranteed balance cap |
| **C2**: Juan Pérez | Natural, Male, Single | SLV | NIT + DUI | Yes, collateral = $5,000 | Tests below-limit guarantee, typical Salvadoran |
| **C3**: John Smith | Natural, Male, Single | USA | Passport only | No | Tests non-resident, single document, no guarantee |
| **C4**: Ana de García | Natural, Female, Married | SLV | NIT + DUI | Yes, collateral = $10,289 | Tests exact-limit guarantee, married name |
| **C5**: Roberto Martínez | Natural, Male, Divorced | SLV | NIT + DUI | Yes, collateral > $10,289 | Tests divorced marital status, multiple accounts |
| **C6**: Elena Ramírez | Natural, Female, Single | MEX | Passport + NIT | No | Tests foreign national with NIT (resident status) |
| **C7**: Carlos Hernández | Natural, Male, Single | SLV | NIT + DUI | Yes, collateral = $0 | Tests customer with facility but no collateral yet |

### Phase 3: Deposit Accounts and Transactions

| Account | Customer | Deposits | Withdrawals | Expected Balance |
|---|---|---|---|---|
| **DA1** | C1 | $8,000 + $4,000 + $3,000 = $15,000 | $2,000 | $13,000 |
| **DA2** | C2 | $5,000 | None | $5,000 |
| **DA3** | C3 | $20,000 | $15,000 | $5,000 |
| **DA4** | C4 | $10,289 | None | $10,289 |
| **DA5** | C5 | $7,500 | None | $7,500 |
| **DA6** | C5 | $3,000 | $1,000 | $2,000 |
| **DA7** | C6 | $1,000 | None | $1,000 |
| **DA8** | C7 | $500 | None | $500 |

*Note: "Dollar amounts" here represent the BTC equivalent at seeding-time price. Actual deposits are in satoshis.*

### Phase 4: Credit Facilities and Collateral

| Facility | Customer | Collateral (BTC) | Collateral (USD @$65k) | Guaranteed Balance |
|---|---|---|---|---|
| **CF1** | C1 | 0.25 BTC | $16,250 | $10,289 (capped) |
| **CF2** | C2 | 0.08 BTC | $5,200 | $5,200 |
| **CF4** | C4 | 0.1584 BTC | $10,289 | $10,289 (exact) |
| **CF5** | C5 | 0.20 BTC | $13,000 | $10,289 (capped) |
| **CF7** | C7 | 0 BTC | $0 | $0 |

---

## Expected Events Summary

### Customer Events (core_customer_events_rollup)

| Event | Count | Trigger |
|---|---|---|
| `CustomerEvent::Initialized` | 7 | One per customer (C1–C7) |
| `CustomerEvent::PersonalInfoUpdated` | 0-7 | If personal info changes after init |
| `CustomerEvent::ActivityUpdated` | 0+ | If any customer is deactivated/reactivated |

### Deposit Account Events (core_deposit_account_events_rollup)

| Event | Count | Trigger |
|---|---|---|
| `DepositAccountEvent::Initialized` | 8 | One per deposit account (DA1–DA8) |

### Deposit Events (core_deposit_events_rollup)

| Event | Count | Trigger |
|---|---|---|
| `DepositEvent::Initialized` | 12 | DA1(3) + DA2(1) + DA3(1) + DA4(1) + DA5(1) + DA6(1) + DA7(1) + DA8(1) + DA3-extra(1) = ~12 |

### Withdrawal Events (core_withdrawal_events_rollup)

| Event | Count | Trigger |
|---|---|---|
| `WithdrawalEvent::Initialized` | 4 | DA1(1) + DA3(1) + DA5(0) + DA6(1) = 3-4 |
| `WithdrawalEvent::Confirmed` | 4 | One per approved withdrawal |

### Credit Facility Events (core_credit_facility_events_rollup)

| Event | Count | Trigger |
|---|---|---|
| `CreditFacilityEvent::Initialized` | 5 | One per facility (CF1, CF2, CF4, CF5, CF7) |
| `CollateralizationStateChanged` | 5+ | At least one per facility (initial collateral deposit) |

### Public IDs (core_public_ids)

| Target Type | Count |
|---|---|
| Customer | 7 |
| Deposit Account | 8 |
| Credit Facility | 5 |

---

## Validation Checklist

After seeding, the following validations from NRSF-03 Art. 15 should pass:

| # | Validation | Expected |
|---|---|---|
| 1 | Sum of saldo_total in DEPOSITOS (active + inactive) + adjustments = COES accounts 2110-2114 | Balanced |
| 2 | Sum of saldo_total for cuenta corriente (cod_prod) + adjustments = COES 211001 | N/A (no corriente product) |
| 3 | Sum of saldo_total for cuenta ahorro + adjustments = COES 211002 + 211003 | N/A (no ahorro product) |
| 4 | Sum of saldo_total for depósito a plazo + adjustments = COES 2111 + 2112 + 2113 - 211202 | N/A (no plazo product) |
| 5 | Sum for certificados + adjustments = COES 211202 | N/A |
| 6 | Sum of restricted/inactive deposits + adjustments = COES 2114 | Should match |

*Note: Since Lana uses a single product type "BTCL" (code genérico = 04, Otros), most traditional product-specific validations don't apply. The total balance validation (#1) is the most critical.*

---

## Cross-Report Consistency Requirements

The norm requires all files to be linked via NIU (field present in Clientes, Documentos, Titulares, Resumen) and Número de cuenta (field present in Depósitos, Titulares, Ajustes):

1. Every NIU in DEPOSITOS must exist in CLIENTES
2. Every NIU in DOCUMENTOS must exist in CLIENTES
3. Every NIU in TITULARES must exist in CLIENTES
4. Every Número de cuenta in TITULARES must exist in DEPOSITOS
5. Every Código del Producto in DEPOSITOS must exist in PRODUCTOS
6. Every Agencia code in CLIENTES must exist in AGENCIAS
7. Número de titulares in DEPOSITOS must match count of TITULARES rows for that account

The seeding plan above ensures all these cross-references are satisfied with 7 customers, 8 deposit accounts, 5 credit facilities, and consistent public ID assignments.

---

## dbt Model Fixes Required

Before seeding data will produce valid reports, the following models need their `1 = 0` filters removed and logic completed:

| Model | Issue | Fix Needed |
|---|---|---|
| `int_nrsf_03_05_agencias` | Disabled, all TODOs | Populate with at least one virtual branch record (could be a dbt seed) |
| `int_nrsf_03_07_funcionarios_y_empleados` | Disabled | Remove `1 = 0`, validate employee data source |
| `int_nrsf_03_08_resumen_de_depositos_garantizados` | Disabled | Remove `1 = 0`, implement per-customer aggregation with balance cap logic |
| `int_nrsf_03_09_ajustes` | Disabled | Remove `1 = 0`, implement reconciliation logic or produce empty output |
