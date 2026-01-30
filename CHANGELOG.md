## [0.39.0] - 2026-01-30

### 🚀 Features

- Change RequireVerifiedCustomerForAccount default to true (#3481)
- *(docs)* Add schema snapshot script for versioning (#3540)
- *(admin-panel)* Display authorized column in audit log table (#3547)
- *(dev)* Add extensible custom lints system (#3558)

### 🐛 Bug Fixes

- Doc generation (#3501)
- Gracefully handle unknown domain config keys at startup (#3516)
- Add timeout to server graceful shutdown (#3499)
- Migration of known complex anyof type (#3568)
- Increase PostgreSQL max_connections to 200 for CI test parallelism (#3578)

### 🚜 Refactor

- Move SumSub credentials to domain config (#3486)
- Custody public events (#3504)
- Deposit and governance public events (#3494)
- Prevent infinite loop in scenarios (#3544)
- Interest late scenario (#3554)
- Core/report public events (#3493)
- Separate collateral ledger from credit ledger (#3552)
- Remove redundant update_pending_credit_facility_collateral (#3566)
- *(core-access)* Encapsulate superuser check in Role entity
- Move liquidation jobs to collateral (#3572)
- Credit module config validation against base (#3424)
- Use ephemeral events for reports (#3582)

### 🧪 Testing

- Report public events (#3561)

### ⚙️ Miscellaneous Tasks

- Release 0.38.0 [ci skip] (#3505)
- Migrate from kaniko to oci-build-task (#3502)
- Update_in_op for internal domain configs (#3503)
- Remove redundant put step (#3507)
- Restructure release job (#3509)
- Update promote pr prefix (#3500)
- Update Keycloak to 26.5.2 (#3523)
- Bump es-entity + async-graphql (#3528)
- Guaranteed default value for domain config (#3511)
- Add rc pr file checker (#3521)
- Use accounting base config (#3415)
- Add PR limit enforcer workflow (#3556)
- Add ignore pr limit label to promote PRs (#3559)
- Record liquidation in collateral (#3548)
- Add lana-review claude skill (#3562)
- Move liquidation ledger operations (#3563)
- Dbt single execution (#3518)
- Some tweaks to lana-review (#3567)
- Bump obix (#3581)
- Improve the lana-review claude skill (#3579)
## [0.38.0] - 2026-01-27

### 🚀 Features

- Add env var to optionally setup domain config during setup (#3487)

### 🐛 Bug Fixes

- Add missing payment tracking UI elements (#3489)
- Simulation script on ci (#3488)

### 🚜 Refactor

- Public events in core/customer (#3434)
- Refactor `chart` command to focus on base configuration (#3443)

### 📚 Documentation

- Update comment for Claude.md

### 🧪 Testing

- Make DummyEvent resilient to concurrent tests

### ⚙️ Miscellaneous Tasks

- Use clock from repo for begin_op (#3490)
- Remove old job references used for migration (#3492)
- Ignore all internal crates for dependabot (#3491)
- Test 11 (#3473)
- Open promote rc PR as draft (#3497)
- Release 0.37.0 [ci skip] (#3498)
- Bump obix
## [0.37.0] - 2026-01-26

### 🚀 Features

- Sumsub empty table and dep mapping (#3438)
- Add documentation site using docusaurus (#3370)
- Introduce eod job and time events (#3406)

### 🐛 Bug Fixes

- Wait for CSV generation job to complete in e2e test (#3450)

### 🚜 Refactor

- *(keycloak-client)* Use Url type for url field validation (#3447)
- Custody to use inbox (#3285)
- Removing core applicant crate (#3465)
- Use EndOfDay event for customer activity status update (#3475)
- Bootstrap to use clock ctrl (#3430)
- Move KYC check configs to domain config system (#3469)

### ⚙️ Miscellaneous Tasks

- Bring back all file reports (#3439)
- Fix release and prep release (#3440)
- Use `prev_version` in prep release script (#3441)
- Upgrade lodash (#3452)
- Upgrade lodash for docusaurus (#3453)
- Move to gotenberg and remove markdown2pdf for pdf creation (#3446)
- Attempting to fix non deterministic behavior (#3454)
- Fix postgres ssl for concourse (#3455)
- Update claude.md (#3456)
- New test (#3457)
- Test 2 (#3458)
- Test 3 (#3460)
- Test 4 (#3461)
- Test 5 (#3462)
- Test 6 (#3463)
- Test 7, making mailcrab optional (#3464)
- Just needing one value for price interval (#3467)
- Fix test 10 (#3470)
- Remove unnecessary config option (#3468)
- Update deny.toml and remove sec warnings (#3474)
- Increase timeout (#3478)
- Remove timeout for completion from disbursal_diff month scenario (#3480)
- Remove timeouts for completion (#3484)
## [0.36.0] - 2026-01-23

### 🚀 Features

- Csv uploaded subscription (#3404)
- *(reports)* Connect runtime to dagster (#3180)

### 🐛 Bug Fixes

- Add missing credit module config account codes to copy (#3400)
- Disable pytest for python313 packages in flake.nix (#3398)
- Dagster lana dw json error in udf (#3395)
- Improve naming to avoid user confusion (#3413)
- Remove dagster from general docker file (#3420)
- Cypress tests (#3419)

### 📚 Documentation

- Domain configs sytem (#3365)

### ⚙️ Miscellaneous Tasks

- Disable the bumping of the dagster init digest (#3391)
- Introduce release candidates and manual releasing (#3388)
- Dagster related fixes (#3394)
- Bump flake (#3392)
- Dg init container cleanup (#3405)
- Add payment allocation job (#3343)
- Configure chart of accounts integrated accounting modules from `Chart` entity (#3386)
- Release 0.36.0 (#3422)
- Trigger build-rc on new dagster img (#3425)
- Fix passed fields on release inputs (#3426)
- Fix release (#3428)
