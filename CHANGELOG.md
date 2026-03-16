## [0.48.0] - 2026-03-16

### 🚀 Features

- *(admin-panel)* Show custodian provider type in list page (#4283)
- *(admin-panel)* Add createdAt column to customers and prospects (#4289)
- Sql models, seeds and yml files for nrp91 (#4293)
- *(admin-panel)* Add system info dialog on version click (#4327)
- *(admin-panel)* Add creditFacilityComplete mutation UI (#4318)
- *(admin-panel)* Add custodianConfigUpdate mutation UI (#4359)
- *(money)* Introduce generic MinorUnits<C> currency types (#4345)
- *(price)* Dynamic price provider configuration (#4362)

### 🐛 Bug Fixes

- *(collateral)* Default manual collateral update config to false (#4279)
- Data pipeline out of date fields (#4288)
- *(admin-panel)* Translate customer page strings to Spanish (#4294)
- *(admin-panel)* Translate configurations page strings to Spanish (#4295)
- *(admin-panel)* Translate miscellaneous admin panel strings to Spanish (#4296)
- *(profit-and-loss)* Compute NET client-side and use currency-aware balance access (#4284)
- *(admin-panel)* Fix Spanish text truncation in audit filter and tables (#4298)
- *(bats)* Move flaky accruals test to integration layer (#4257)
- *(ci)* Cap outbox-dependent test parallelism to fix flaky test-integration (#4299)
- *(docs)* Lingo.dev producing hallucinated outputs (#4313)
- *(admin-panel)* Display error on liquidations page when unauthorized (#4287)
- *(customer-server)* Use CvlPct union type instead of scalar for CVL percentages (#4291)
- *(admin-panel)* Use generic error message on pending credit facilities page (#4286)
- Resolve pnpm audit vulnerabilities (#4328)
- *(deposit)* Standardize chartOfAccounts field naming in deposit config (#4292)
- *(test)* Make accrual history assertion resilient to stale job interference (#4333)
- *(docs)* Simplify translation scripts — remove walkthrough snapshot mechanism (#4329)
- Check if facility active for payments/disbursals (#4304)
- *(credit)* Replace RescheduleIn(5 min) with RescheduleNow in process_accrual_cycle (#4379)

### 🚜 Refactor

- *(governance)* Extract publisher for approval process conclusion (#4290)
- *(cli)* Remove sim-bootstrap compile-time feature flag (#4285)
- *(credit)* Liquidation flow — thin handler + command jobs (#4316)
- Rename loanAgreementGenerate to creditFacilityAgreementGenerate (#4325)
- *(config)* Rename domain config keys for consistency (#4317)
- Remove allow-manual-custodian config (#4322)
- *(governance)* Rename SystemAutoApprove to AutoApprove (#4347)
- *(graphql)* Use TermsInput in TermsTemplateCreate/UpdateInput for consistency (#4356)
- Remove bats/lana-normal.yml in favor of --set override (#4367)
- Remove sumsub-testing feature flag (#4369)
- *(credit)* Parameterize currency in CEL templates (#4363)
- *(deposit-sync)* Sumsub export — thin handler + command jobs (#4326)
- Replace Utc::now() with clock-based time in custody webhook and contract creation (#4377)
- Remove unused JsonSchema derives from internal types (#4384)

### 📚 Documentation

- Frontend-skills for claude (#4314)

### 🧪 Testing

- Create manual custodian only once in cypress (#4300)

### ⚙️ Miscellaneous Tasks

- Release 0.47.0 [ci skip] (#4281)
- Turn manual custodian into regular custodian (#4268)
- *(docs)* Remove code coverage documentation (#4309)
- Upgrade quinn-proto (#4323)
- *(test)* Switch BATS clock from auto_advance to manual (#4360)

### ◀️ Revert

- "fix(ci): cap outbox-dependent test parallelism to fix flaky test-integration" (#4299) (#4303)
## [0.47.0] - 2026-03-10

### 🚀 Features

- *(customer)* Handle SumSub applicantOnHold callback (#4142)
- Add column sorting to admin panel tables (#4107)
- *(customer)* Add frozen status, close customer, and SumSub deactivation (#4112)
- Improve P&L date range selection with fiscal period presets (#4039)
- Add sorting for Generated At column in regulatory reporting (#4156)
- *(customer)* Make SumSub KYC/KYB flow names configurable (#4164)
- Add active filter chips to paginated table (#4130)
- *(governance)* Simplify committee approval to all-must-approve (#4167)
- *(reporting)* As-of date report generation (#3949)
- *(governance)* Require at least one member when creating a committee (#4180)
- *(customer)* Capture company name from SumSub KYB verification (#4186)
- Add pagination and sorting to users list (#4189)
- Add pagination and sorting to roles list (#4190)
- *(lints)* Add service-conditionals lint rule (Tell, Don't Ask) (#4183)
- *(customer)* Add CustomerConversion enum to track conversion reason (#4204)
- *(admin-panel)* Add customer freeze/unfreeze UI (#4207)
- *(governance)* Add domain config to prevent auto-approve for policies (#4168)
- *(lints)* Add tainted-transaction-use lint rule (#4216)
- *(credit)* Add domain config to disable manual collateral (#4155)
- *(admin-panel)* Add input bounds and CVL ordering validation (#4235)
- *(customer-sync)* Sync status with SumSub (#4209)
- *(domain-config)* Add post_hydrate_hook to validate encrypted conf… (#4233)
- *(docs)* Integrate api/event description translation into lingo.dev workflow (#4187)
- *(admin-server)* Expose entity event history on GraphQL (#4234)
- Event history improvements — expandable details, new entities, shared fragment (#4254)
- *(admin-panel)* Add programmatic range validation for term fields (#4256)
- Improve manual regulatory report generation ui (#4230)
- *(custody)* Add self-custody bitcoin custodian (#4249)

### 🐛 Bug Fixes

- *(credit)* Handle NoAccrualCycleInProgress in accrue_period to prevent infinite retry (#4148)
- *(deps)* Override dompurify to fix Dependabot alert #202 (#4150)
- *(collection)* Downgrade obligation ConcurrentModification error severity to WARN (#4153)
- *(customer)* Reduce false-positive alerts in KYC callback handling (#4151)
- *(docs)* Correct Spanish version labels showing "Siguiente" instead of version numbers (#4159)
- Correct deposits page description (#4166)
- *(cli)* Check port availability before running initialization (#4188)
- *(ci)* Resolve concurrency race between translation workflows (#4193)
- *(admin-panel)* Add missing sumsub flow name translations (#4195)
- *(admin-panel)* Fix journal pagination and add cache policies (#4196)
- *(governance)* Handle race condition in bootstrap_default_committee
- *(governance)* Remove dead duplicate detection in From<sqlx::Error>
- *(governance)* Use _in_op repo method in bootstrap_default_committee
- *(customer)* Correct verification semantics and rename config flag (#4217)
- *(customer)* Reset decline_kyc idempotency guard after KYC approval (#4211)
- *(docs)* Add missing event modules to versioned docs and fix Spanish placeholder (#4244)
- *(admin-panel)* Remove monthly payment field from credit facility details (#4250)
- *(credit)* Use Date scalar for repayment due_at to fix off-by-one display (#4251)
- Effective rate calculation (#4239)
- *(admin-panel)* Filter superuser role from role assignment dropdown (#4263)
- *(deposit)* Ignore internal status postings for inactivity (#4255)
- *(admin-server)* Move reason param inside ApprovalProcessDenyInput (#4273)
- *(admin-server)* Rename customerDocumentAttach to customerDocumentCreate (#4274)
- *(cli)* Make env-var args named to prevent swallowing subcommands (#4267)
- *(admin-panel)* Hide action buttons when user lacks write permissions (#4264)

### 🚜 Refactor

- *(customer)* Remove ProspectStatus in favor of ProspectStage (#4170)
- *(customer)* Remove KycVerification in favor of applicant_id (#4184)
- *(deposit)* Remove DepositAccountStatus::Inactive and holder status machinery (#4205)
- *(customer)* Move ManualConversionNotAllowed guard into prospect entity (#4225)
- *(report)* Remove redundant reports_bucket_name config (#4265)
- *(admin-server)* Scope EsEntity trait import to event_history fns
- *(graphql)* Rename prospect KYC link mutation (#4271)

### 📚 Documentation

- Update documentation to recent changes (#4103)

### 🧪 Testing

- Test for default value in domain config (#4109)

### ⚙️ Miscellaneous Tasks

- Release 0.46.1 [ci skip] (#4145)
- Update apollo client for sort (#4158)
- Update staging URLs from staging.lana.galoy.io to staging.galoy.io (#4176)
- Move checking account activity from customer to deposit account (#4175)
- Set default value for as-of reports, raise if no value (#4197)
- Update customer page layout (#4220)
- Expose canonical creditFacilityId in pending facility flow (#4208)
- *(admin-panel)* Add missing translations and misc improvements (#4226)
- Update es-entity to 0.10.26 and migrate idempotency_guard! to named params (#4232)
- Update CommandMenu text (#4238)
- *(bitgo)* Ignore simulated transfer notifications (#4245)
- Switch maturity job to use time events (#3996)
- Add frontend checks to flake (#4247)
- Update nix-cache
- *(codegen)* Sort GraphQL schema fields alphabetically (#4272)
## [0.46.1] - 2026-03-05

### 🐛 Bug Fixes

- Env var secret leakage in logs (#4121)
- Increase admin panel session timeout from 5 to 30 minutes (#4133)
- Replace .expect() with error returns in interest accrual cycle methods (#4118)
- Replace derive(Debug) with custom impl on EncryptionKey to redact key material (#4137)
- *(credit)* Handle NoNextAccrualPeriod in accrue_period to prevent infinite retry (#4141)
- Replace derive(Debug) with custom impl on SumsubClient and SumsubConfig to redact secret (#4139)
- Replace derive(Debug) with custom impl on BitgoClient to redact secrets (#4138)
- *(deps)* Address all open Dependabot security alerts (#4113)

### 🚜 Refactor

- *(customer)* Remove legacy event variants from Customer and Prospect (#4119)
- Remove ChartOfAccountsAddRootNode mutation (#4129)

### ⚙️ Miscellaneous Tasks

- Release 0.46.0 [ci skip] (#4120)
- Remove leftover hakari config file (#4135)
## [0.46.0] - 2026-03-05

### 🚀 Features

- Add customer type filtering for customers and prospects (#4077)
- Auto-translate docs via Lingo.dev with symlink wrapper (#3965)

### 🐛 Bug Fixes

- Update deps to fix audit (#4110)
- Downgrade ConcurrentModification severity from ERROR to WARN (#4111)

### 🚜 Refactor

- Point in time balance sheet query (#4065)
- Update es-entity, cala-ledger, job, and obix dependencies (#4097)
- Rename accounting-primitives to chart-primitives (#4016)

### ⚙️ Miscellaneous Tasks

- Release 0.45.0 [ci skip] (#4101)
- Switch interest accrual job to use time event (#3997)
## [0.45.0] - 2026-03-04

### 🚀 Features

- Add status filtering to proposals and pending facilities (#4054)
- Add --set flag for CLI config overrides (#3968)
- Expose build version info via admin GraphQL API (#4052)
- Add collateralization state filter to pending credit facilities (#4070)
- Add status filtering to disbursals list   (#4056)
- Add status filtering for deposits and withdrawals (#4071)
- Add status filtering for deposit accounts (#4074)
- Add column sorting to proposals and pending facilities (#4066)

### 🐛 Bug Fixes

- Standardize admin GraphQL ID contract (#4035)
- Flaky cypress customer tests (#4008)
- Downgrade SumSub 404 error severity from ERROR to WARN (#4047)
- Pagination (#4049)
- Use consistent AuditEntry global ID prefix (#4063)
- Standardize GraphQL ID naming for all entity types (#4038)
- Prospect list query (#4053)
- Downgrade FiscalYearError severity for business validation errors (#4073)
- Pagination for filtered list (#4080)
- *(deposit)* Downgrade ExternalIdAlreadyExists severity from ERROR to INFO (#4083)
- Consistent domain config values in ui (#4086)
- Use balance sheet specific loader (#4055)
- Improve date/timestamp handling in GraphQL layer (#4092)
- *(customer)* Map party DB constraint violations to correct error variants (#4098)
- Downgrade VelocityError::Enforcement severity from ERROR to WARN (#4099)

### 🚜 Refactor

- Break up 3 event handlers into handler + command job (#4010)
- Break up 4 more event handlers into handler + command job (#4044)
- *(admin-server)* Remove duplicate build.rs, inject build info from CLI at runtime (#4078)
- Rename subscription stream variables for clarity (#4076)
- Make customer_type non-optional in Customer and Prospect entities (#4090)
- *(admin-server)* Scope DataLoader per-request instead of globally (#4075)

### ⚙️ Miscellaneous Tasks

- Allow netlify commit alias deploy to fail gracefully (#4017)
- Release 0.44.0 [ci skip] (#4020)
- Downgrade error to warn in OTEL (#4001)
- Symlink .agents/skills to .claude/skills for Codex compatibility (#4025)
- Strip symbols from release binary (#4029)
- Add claude docs skills (#3707)
- Improve cypress test reliability (#4045)
- Add lana-multi-commit skill for staged commit workflows (#4064)
- Remove aws-lc-sys from dependency tree (#4068)
- Change bq strategy (#4051)
- Add lana-alert-fixer skill for investigating and fixing alerts (#4081)
## [0.44.0] - 2026-02-27

### 🚀 Features

- Add `make seed-data` to populate DB and exit (#3955)
- *(collection)* Migrate obligation lifecycle to EndOfDay batch processing (#3926)
- *(dev)* Add Jaeger and OpenTelemetry MCP server to local dev stack (#3994)
- Update and introduce key rotation in domain-config  (#3927)

### 🐛 Bug Fixes

- Use event-driven waiting in create_pending_facility test helper (#3945)
- Disable animations in Cypress tests to prevent flaky inputs (#3987)
- Validate effective dates for credit facility payments (#3699)
- *(ci)* Add --no-build to Netlify commit alias deploy (#3991)
- Pnpm audit (#4009)

### 💼 Other

- Selective list_for(by(...)) for compile time reduction (#3939)

### 🚜 Refactor

- Introduce CoreCreditCollateralEvent (#3941)
- Inline seed_only check into bootstrap block (#3969)
- Extract core-accounting-primitives to break serial compilation chain (#3918)
- Break up event handler into handler + command job (#3869)
- Extract collatateral crate (#3984)

### 📚 Documentation

- Accounting and product module es coverage (#3963)

### ⚡ Performance

- Use _in_op repo methods when transaction already in scope (#3986)

### 🧪 Testing

- AccrualPosted event (#3923)
- FacilityCollateralizationChanged event (#3967)

### ⚙️ Miscellaneous Tasks

- Release 0.43.0 [ci skip] (#3961)
- Introduce CoreCreditCollateral Action and Object (#3954)
- Bump cala-* to 0.13.19 (#3972)
- Switch to pnpm for docs (#3992)
- Make product module COA config immutable (#3932)
- Fix pnpm run syntax in update docs (#3995)
- Upgrade storybook (#4004)
- Lint mandatory in op (#4000)
- Bump es-entity / use soft_without_queries
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
