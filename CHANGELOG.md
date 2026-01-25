## [0.37.0] - 2026-01-25

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
