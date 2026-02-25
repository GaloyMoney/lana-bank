## [0.44.0] - 2026-02-25

### 🚀 Features

- Add `make seed-data` to populate DB and exit (#3955)

### 🐛 Bug Fixes

- Use event-driven waiting in create_pending_facility test helper (#3945)

### 💼 Other

- Selective list_for(by(...)) for compile time reduction (#3939)

### 🚜 Refactor

- Introduce CoreCreditCollateralEvent (#3941)
- Inline seed_only check into bootstrap block (#3969)
- Extract core-accounting-primitives to break serial compilation chain (#3918)

### 📚 Documentation

- Accounting and product module es coverage (#3963)

### 🧪 Testing

- AccrualPosted event (#3923)
- FacilityCollateralizationChanged event (#3967)

### ⚙️ Miscellaneous Tasks

- Release 0.43.0 [ci skip] (#3961)
- Introduce CoreCreditCollateral Action and Object (#3954)
- Bump cala-* to 0.13.19 (#3972)
## [0.43.0] - 2026-02-24

### 🚀 Features

- Add configurable price providers to Price module (#3929)
- Add lana-build-compare skill (#3940)

### 🐛 Bug Fixes

- Use _in_op variants to avoid connection pool amplification (#3901)
- Install rustls crypto provider to prevent panic in google-cloud crates (#3924)
- Override ajv@<6.14.0 to resolve Dependabot alert #193 (#3936)
- *(ci)* Normalize Cypress retry screenshot filenames before manifest generation (#3931)
- Serialize core-price and core-credit tests to prevent flaky NOTIFY race (#3948)
- Use event-driven waiting in overpayment integration test (#3943)

### 🚜 Refactor

- Use shared mod cfg update dialog component (#3848)
- Remove references to CollateralRepo from non-collateral crates (#3876)

### 📚 Documentation

- Accounting and product module account sets (#3875)

### 🧪 Testing

- E2e report download link via GCS in data-pipeline CI (#3950)

### ⚙️ Miscellaneous Tasks

- Release 0.42.0 [ci skip] (#3925)
- *(nix)* Use --all-features for lana-deps derivation (#3935)
- Run build-compare builds twice, keep fastest (#3947)
- Add repo cache clearer (#3953)
- Make doc generation script ci safe (#3957)
## [0.42.0] - 2026-02-23

### 🚀 Features

- Add local file serving endpoint with signed URLs (#3788)
- Add collateralization data to CreditFacilityEvent::Initialized (#3825)
- Split customer module into prospect + customer (#3778)
- Multiasset file reports (#3847)

### 🐛 Bug Fixes

- Proper conditions for cold start (#3826)
- Customer creation in tests (#3843)
- Use event stream instead of polling in create_active_facility test helper (#3849)
- Resolve high-severity pnpm audit vulnerabilities (#3855)
- Override minimatch to >=10.2.1 (CVE-2026-26996) (#3856)
- *(docs,ci)* Fix broken screenshot refs and reset DB between Cypress locale runs (#3862)
- Make Collateral.creditFacility nullable to prevent panic (#3857)
- Misc data issues on pipeline (#3866)
- *(ci)* Use pre-built binary for English Cypress run server restart (#3877)
- Bring latest cala_ledger_setup
- *(ci)* Remove DB reset for English Cypress run, use ES screenshots as fallback (#3879)
- Hakari
- *(deps)* Update keccak 0.1.5 -> 0.1.6 to fix Dependabot alert #190 (#3886)
- Minimatch and ajv deps in docs (#3890)
- Remove double generation of LiquidationId (#3871)
- Propagate json-schema flag (#3898)
- *(docs)* Correct GitHub navbar link and remove changelog (#3889)

### 🚜 Refactor

- Deposit product mod `AccountSet` as value including accounting context (#3745)
- Migrate to es-entity multi-filter list_for_filters API (#3833)
- Use lib/encryption in domain config (#3829)
- Facility collateralization changed event (#3798)
- Separate collateral and facility ledger concerns (#3763)
- *(admin-panel)* Replace refetchQueries with cache updates via fragments (#3846)
- PendingCreditFacilityCollateralizationChanged public event (#3842)
- Use OutboxEventHandler for all outbox job handlers (#3831)
- Remove console-subscriber to eliminate protobuf-src compile cost (#3874)
- Use lib/encryption for custody (#3840)
- Move liquidation jobs listening to credit events to credit (#3852)
- Use es-entity update_all for batch entity persistence (#3894)
- Introduce SecuredLoanId (#3854)

### 📚 Documentation

- Improve docs v2 (#3771)
- Rewrite release engineering documentation (#3863)

### 🧪 Testing

- Credit public events (#3824)
- PendingCreditFacilityCollateralizationChanged event (#3864)

### ⚙️ Miscellaneous Tasks

- Release 0.41.1 [ci skip] (#3832)
- Reengage liquidation-related public events (#3807)
- Credit COA config update (UI) (#3748)
- Deposit COA config update (UI) (#3753)
- Bump flake (#3880)
- Bump cala
- Bump cala
- Set is-release file in chart (#3891)
- Fix cala-ledger double compilation via hakari config (#3892)
- Remove aws-lc-sys from dependency tree (#3885)
- Minor cleanup in encryption use-cases (#3893)
- Remove lto from release profile (#3897)
- Exclude cala crates from cargo hakari (#3902)
- Use pnpm instead of npm in docs-site (#3909)
- Remove cargo hakari and workspace-hack (#3911)
- Upgrade async-graphql from 7.2 to 8.0.0-rc.3 (#3905)
## [0.41.1] - 2026-02-17

### 🐛 Bug Fixes

- Cypress stack cleanup and stop-cypress-stack target (#3828)

### ⚙️ Miscellaneous Tasks

- Release 0.41.0 [ci skip] (#3823)
- Reintroduce collateral accounts in Credit Faciliy GQL (#3827)
## [0.41.0] - 2026-02-16

### 🚀 Features

- Type-safe encrypted domain config (#3769)
- Translate transaction template codes in admin panel (#3784)
- Add event type to data pipeline rollups (#3787)
- Start root assets on first deploy (#3791)
- Bypass authz check for system actors (#3819)

### 🐛 Bug Fixes

- Collapse nested if-let to satisfy clippy collapsible_if lint (#3792)
- Exclude workspace-hack from dependabot cargo scans (#3804)
- Restore workspace-hack from base branch before hakari regeneration (#3809)
- Suppress hydration warning on html tag in admin panel (#3803)
- Apply startup domain configs before app initialization (#3817)

### 🚜 Refactor

- Move codegen binaries to dev/codegen crate (#3781)
- Credit product mod `AccountSet` as value including accounting context (#3741)
- Core-credit public events (#3719)

### 🧪 Testing

- Test json updates in domain config (#3800)

### ⚙️ Miscellaneous Tasks

- Adding fly to flake (#3756)
- Create skills for testing PR on staging (#3764)
- Translate journal transaction descriptions (#3768)
- Removing claude code review (#3793)
- Use encrypted flag for sumsub and update ui (#3797)
## [0.40.0] - 2026-02-11

### 🚀 Features

- Add dagster e2e bats test (#3654)
- *(docs)* Replace local search with Algolia DocSearch (#3751)
- Differentiate system actors in audit layer (#3720)

### 🐛 Bug Fixes

- Autofill values for credit module (#3687)
- Core_money -> money (#3702)
- *(domain-config)* Reduce DuplicateKey error severity to DEBUG (#3704)
- Docs not showing the latest cut version (#3710)
- Bump axios to 1.13.5 to resolve DoS vulnerability (GHSA-43fc-jf86-j433) (#3721)
- *(accounting)* Include leaf account sets in descendant_account_sets() (#3717)
- *(test)* Eliminate flaky payment allocation race condition (#3723)
- *(deps)* Upgrade google-cloud-storage to fix jsonwebtoken vulnerability (#3724)
- *(deps)* Regenerate entity-rollups lockfile to fix 3 vulnerabilities (#3737)
- *(deps)* Update job crate to 0.6.4 (#3739)
- Remove stale event_jobs_board_id output (#3742)
- Refactor model to simplify final query (#3731)

### 🚜 Refactor

- Extract core utilities to lib and add custom lint (#3688)
- Move collateral accounts (#3650)
- Complete simplification of nested `Liquidation` entity (#3661)
- Return cursor in Liquidation-related queries (#3747)
- Centralize domain config test cleanup into DomainConfigTestUtils (#3706)

### 📚 Documentation

- Improve documentation (#3705)

### 🧪 Testing

- Collection public events (#3669)
- Add feature-gated GCS integration tests (#3735)

### ⚙️ Miscellaneous Tasks

- Release 0.39.0 [ci skip] (#3684)
- Fix release and update docs (#3690)
- Update nix flake inputs (#3692)
- Temp fix dagster tests (#3701)
- Remove status in ci
- Add lana-agent-self-improve skill for claude (#3714)
- Add lana-monitor-pr-actions skill for CI monitoring (#3711)
- Combine liquidation completion into proceeds recording (#3664)
- Schedule dbt model executions (#3709)
- Remove redundant collateral update (#3679)
- Refine claude skill files for CI monitoring and test writing (#3727)
- Do not expose LiquidationError beyond Collateral (#3746)
- Add cargo fmt Claude Code hook for auto-formatting (#3729)
- Moving indempotent instruction to a skill (#3754)
- Add Algolia site verification meta tag (#3759)
## [0.39.0] - 2026-02-06

### 🚀 Features

- Change RequireVerifiedCustomerForAccount default to true (#3481)
- *(docs)* Add schema snapshot script for versioning (#3540)
- *(admin-panel)* Display authorized column in audit log table (#3547)
- *(dev)* Add extensible custom lints system (#3558)
- *(audit)* Add filters to audit list query (#3550)
- Add GraphQL query for off-balance sheet account sets (#3635)

### 🐛 Bug Fixes

- Doc generation (#3501)
- Gracefully handle unknown domain config keys at startup (#3516)
- Add timeout to server graceful shutdown (#3499)
- Migration of known complex anyof type (#3568)
- Increase PostgreSQL max_connections to 200 for CI test parallelism (#3578)
- Use bulk import (CSV) to expand existing chart of accounts (#3522)
- Date-only string format (#3632)
- Lastest backend changes to rollups (#3651)
- Configured credit `AccountSet`s skipped in COA attachment (#3641)
- Dagster report sync bugs (#3674)

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
- Use ephemeral for export-csv (#3596)
- Extract terms crate (#3506)
- Move disbursal and proposal completion checks to TermValues (#3608)
- *(credit)* Remove dead code from Liquidations service (#3587)
- Mv deposit coa config from ledger to `InternalDomainConfigs` (#3593)
- Extract collection crate from obligation, payment and payment allocation (#3619)
- Derive off balance sheet parent account codes (#3617)
- Simplify module visibility in credit collection (#3638)
- Credit/collection public event (#3642)
- Remove credit_facility FK from obligation, payment, payment_allocation tables (#3672)
- Some improvements to collateral module (#3668)
- Move account id from liquidation entity (#3649)

### 📚 Documentation

- Add release process docs (#3520)
- Reorganise docs  (#3538)

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
- Move liquidation-sent to collateral use case (#3570)
- Liquidation in collateral entity (#3573)
- Consolidate if blocks clippy error (#3597)
- Update docs in rc promotion pr (#3571)
- Fix update docs derivation (#3605)
- Fix update docs deps (#3606)
- Fmt `ci/flake.nix` (#3607)
- Ignore consts typo from typenum crate (#3611)
- Replace `liquidation_id` with `collateral_id` in input (#3574)
- Move proceeds use case to collateral (#3585)
- Add transaction_commit lint (#3612)
- Add db-op-convention lint and fix all violations
- Add db-op-convention lint and fix all violations (#3622)
- AccountSetsByCategory query with AccountCategory enum (#3601)
- Lint unwrap usage in prod code (#3634)
- Add idempotent lint (#3629)
- Entity query lint (#3644)
- Constructor naming lint (#3656)
- Add lana-test-writer claude skill (#3633)
- Nest `Liquidation` within `Collateral` entity (#3630)
- Deploy docs on tag push (#3553)
- Add created internal credit account sets to config (#3646)
- Remove dead code from collection migration (#3678)
- Add Claude Code GitHub Workflow (#3648)
- Testing with sticky comment (#3682)

### ◀️ Revert

- "ci: add db-op-convention lint and fix all violations"
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
