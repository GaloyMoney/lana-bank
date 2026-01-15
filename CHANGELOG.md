## [0.38.0] - 2026-01-15

### ⚙️ Miscellaneous Tasks

- Use git cliff for changelog generation
## [0.38.0-rc.2] - 2026-01-15

### ⚙️ Miscellaneous Tasks

- Test
## [0.38.0-rc.1] - 2026-01-15

### 🚀 Features

- Test
- Test
- Test

### ⚡ Performance

- Test 3
- Test

### ⚙️ Miscellaneous Tasks

- Test
- Temp update
- More refactor
- Add promote pr job
- Make task executable
- Test
- Test
- Test 2
- Update
- Updates
- Update
- Update
## [0.37.0] - 2026-01-09

### 🚀 Features

- Test

### ⚙️ Miscellaneous Tasks

- Update `ci/flake.lock` (#3314)
- Add `next-rc-version` derivation
- Add jobs
- Dummy commit
## [0.30.3] - 2026-01-07

### ⚙️ Miscellaneous Tasks

- Optimize dg run bootstrap (#3312)
## [0.30.1] - 2026-01-06

### ⚙️ Miscellaneous Tasks

- Post ledger transactions during liquidations (#3216)
## [0.30.0] - 2026-01-06

### 🚀 Features

- Generic UI for exposed config (#3299)
## [0.29.3] - 2026-01-06

### ⚙️ Miscellaneous Tasks

- Change the polling params of test (#3303)
- Make udf creation skippable (#3307)
## [0.29.2] - 2026-01-06

### ⚙️ Miscellaneous Tasks

- Add sql formater (#3294)
## [0.29.1] - 2026-01-05

### ⚙️ Miscellaneous Tasks

- Silence false positive bitfinex alert (#3304)
## [0.29.0] - 2026-01-05

### 🚀 Features

- Dagster lana dw dbt step three clean (#3253)
## [0.28.0] - 2026-01-02

### 🚀 Features

- InitiatedBy in LedgerTransaction (#3284)

### 🚜 Refactor

- Remove unused jobs field (#3293)
## [0.27.2] - 2025-12-31

### 🐛 Bug Fixes

- *(reporting)* Use inbox_events table for sumsub extract and load (#3288)

### ⚙️ Miscellaneous Tasks

- Add `record payment` ledger transaction (#3282)
## [0.27.1] - 2025-12-30

### ⚙️ Miscellaneous Tasks

- Skip dagster tests in concourse (#3283)
- Update tailwind to v4 (#3267)
- Fix pnpm audit (#3289)
## [0.27.0] - 2025-12-30

### 🚀 Features

- Simple domain configs (#3260)

### 🚜 Refactor

- Simplify `ObligationAccounts` type (#3280)
## [0.26.14] - 2025-12-29

### 🚜 Refactor

- Inject account for record payment (#3279)
- Applicant inbox handling (#3281)
## [0.26.13] - 2025-12-29

### 🐛 Bug Fixes

- Wrong obligation account used from interest accruals (#3278)
## [0.26.12] - 2025-12-27

### 🚜 Refactor

- Use inbox for handling sumsub callbacks (#3275)
## [0.26.9] - 2025-12-26

### 🐛 Bug Fixes

- Ignore NotFound in sandbox webhooks (#3272)

### ⚙️ Miscellaneous Tasks

- Merging jobs boards (#3273)
## [0.26.7] - 2025-12-24

### 🐛 Bug Fixes

- Closing tx satisfies cel conditions after monthly root account set metadata update (#3257)
- Make maybe_fire_concluded_event atomic (#3269)
## [0.26.6] - 2025-12-23

### ⚙️ Miscellaneous Tasks

- Add new FiscalYearConfig (#3255)
## [0.26.5] - 2025-12-22

### ⚙️ Miscellaneous Tasks

- Record and allocate payment from liquidation (#3186)
- Add Liquidation to Credit Facility history (#3259)
## [0.26.4] - 2025-12-22

### ⚙️ Miscellaneous Tasks

- Add seeds to dagster (#3258)
- Restructure job boards (#3265)
- Release lock on failure of test jobs (#3266)
## [0.26.3] - 2025-12-20

### ⚙️ Miscellaneous Tasks

- Use obix / bump cala (#3256)
## [0.26.1] - 2025-12-19

### 🐛 Bug Fixes

- *(dev)* Keycloak health check (#3252)
- Month not ended pre-condition (#3251)

### ⚙️ Miscellaneous Tasks

- Email notifications for partial liquidation (#3223)
- Fiscal year UI improvements (#3230)
- Bats test for partial liquidation (#3233)
## [0.26.0] - 2025-12-18

### 🚀 Features

- Dagster lana dw dbt intermediate tables (#3227)
## [0.25.0] - 2025-12-17

### 🚀 Features

- *(reporting)* Sumsub extract and load with dlt (#3206)
## [0.24.12] - 2025-12-17

### 🚜 Refactor

- Split ClosingSpec type (#3248)
## [0.24.11] - 2025-12-17

### ⚙️ Miscellaneous Tasks

- Add back db event (#3244)
## [0.24.10] - 2025-12-17

### ⚙️ Miscellaneous Tasks

- Override systeminformation version (#3242)
## [0.24.9] - 2025-12-17

### ⚙️ Miscellaneous Tasks

- Remove old artifacts related to liquidation (#3236)
## [0.24.8] - 2025-12-16

### 🐛 Bug Fixes

- Use public id for url (#3240)

### ⚙️ Miscellaneous Tasks

- Trigger build when starting deps (#3238)
- Deactivate slow statements (#3241)
## [0.24.7] - 2025-12-16

### ⚙️ Miscellaneous Tasks

- Closing transaction - net income from `ProfitAndLossStatement` to `BalanceSheet` (#3188)
- Increasing log slow statement time (#3239)
- Simplify UTC fetching (#3237)
## [0.24.6] - 2025-12-16

### 🚜 Refactor

- Prioritize shutdown in jobs (#3232)
## [0.24.5] - 2025-12-15

### 🐛 Bug Fixes

- Correct trace param name (#3229)
## [0.24.3] - 2025-12-12

### ⚙️ Miscellaneous Tasks

- Limit dependabot PR to once a week (#3225)
## [0.24.2] - 2025-12-12

### ⚙️ Miscellaneous Tasks

- Improve build speed (#3211)
- Remove old file reference (#3219)
- Fix build following use of lld linker (#3220)
- New attempt at fixing build (#3221)
- Test new build (#3222)
## [0.24.0] - 2025-12-12

### 🚀 Features

- *(admin-panel)* Liquidation UI (#3217)

### 🐛 Bug Fixes

- Liquidations job not running (#3214)

### ⚙️ Miscellaneous Tasks

- Remove meltano and airflow (#3210)
- Make notification email domain config (#3181)
- Re-add tracing err to BoxStream returning functions (#3205)
## [0.23.1] - 2025-12-11

### ⚙️ Miscellaneous Tasks

- Liquidation GQL (#3199)
- Board for jobs with concurrent modification errors (#3197)
- Ignore rust deps for upgrades  (#3207)
- Change gql error tracing to 'warn' (#3204)
- Remove debug build (#3209)
- Unused directive (#3201)
## [0.23.0] - 2025-12-11

### 🚀 Features

- Add dbt lana data warehouse (#3082)
## [0.22.1] - 2025-12-10

### 🐛 Bug Fixes

- DueAt should be EOD (#3167)

### 🧪 Testing

- Check macro for early and explicit returns (#3190)

### ⚙️ Miscellaneous Tasks

- Implement ErrorSeverity for es_entity::EntityError (#3171)
- Remove weird code path for local dev - should use local provider (#3184)
- Implement OutboxError & annotate lib/outbox functions (#3152)
- Add tracing for create_customer fn (#3189)
- Relax ticker schedule (#3194)
- Unused dev dependency (#3195)
## [0.22.0] - 2025-12-08

### 🚀 Features

- *(admin-panel)* Add dialogs for closing the fiscal year and opening the next fiscal year (#3178)
## [0.21.0] - 2025-12-08

### 🚀 Features

- Introduce partial liquidation (#3099)
## [0.20.3] - 2025-12-08

### ⚙️ Miscellaneous Tasks

- Add record_error_severity annotation to lib/komainu (#3172)
## [0.20.0] - 2025-12-05

### 🚀 Features

- `FiscalYear` lifecycle (#3102)

### 🐛 Bug Fixes

- Add ledger txn for denying a withdrawal (#3138)

### ⚙️ Miscellaneous Tasks

- Add severity annotation (#3149)
- Add manual transaction severity annotations (#3153)
- Feat should bump_patch
- Remove facility_cvl from CreditFacility (#3161)
- *(admin-panel)* Move manual transaction creation to ledger-transactions (#3158)
- Add schemas for missing events (#3160)
- Replace Nix/Meltano sqlfluff with lightweight alternative (#3174)
## [0.19.0] - 2025-12-03

### 🚀 Features

- Domain config (#3130)
## [0.18.4] - 2025-12-02

### ⚙️ Miscellaneous Tasks

- Fiscal year UI (#3139)
## [0.18.3] - 2025-12-02

### ⚙️ Miscellaneous Tasks

- Bump job crate
## [0.18.2] - 2025-12-01

### ⚙️ Miscellaneous Tasks

- Verify back lana.yml is up to date (#3140)
## [0.18.0] - 2025-12-01

### 🚀 Features

- Implement error severity trait (#3124)
## [0.17.5] - 2025-11-30

### ⚙️ Miscellaneous Tasks

- Update job (#3136)
## [0.17.4] - 2025-11-28

### ⚙️ Miscellaneous Tasks

- Plug option to debug tokio threads (#3119)
## [0.17.3] - 2025-11-28

### ⚙️ Miscellaneous Tasks

- Update to job 0.2.12 (#3135)
## [0.17.2] - 2025-11-28

### ⚙️ Miscellaneous Tasks

- Clean up severity proc macro (#3131)
- Bump flake (#3133)
## [0.17.1] - 2025-11-27

### 🐛 Bug Fixes

- Add granularity for smoother graph in price board (#3123)
## [0.17.0] - 2025-11-27

### 🚀 Features

- *(data-pipeline)* Report generation in dagster (#3111)
- Add severity proc macro (#3118)

### ⚡ Performance

- Set delay to 5 ms for gql loader to batch better (#3129)

### ⚙️ Miscellaneous Tasks

- Cargo hakari generate workflow over cargo dependabot PR events (#2976)
- Freeze and unfreeze entries in deposit account history (#3128)
- Remove unused file (#3126)
## [0.16.0] - 2025-11-27

### 🚀 Features

- Add ErrorSeverity trait to tracing-utils (#3115)

### ⚡ Performance

- Make op sequential to prevent exhausting connection pool (#3125)

### ⚙️ Miscellaneous Tasks

- Missing price update (#3117)
- Do not instrument run it's not needed (#3116)
## [0.15.2] - 2025-11-26

### ⚙️ Miscellaneous Tasks

- *(deps)* Add cooldown for dependabot updates (#3105)
- Cleanly shutdown permanent jobs (#3114)
## [0.15.0] - 2025-11-25

### 🚀 Features

- Credit facility query for board (#3107)

### 🐛 Bug Fixes

- Flake check

### ⚙️ Miscellaneous Tasks

- Tracing for credit facilities (#3097)
- Add price board (#3106)
## [0.14.1] - 2025-11-25

### 🐛 Bug Fixes

- Chart entity consistency (#3103)
## [0.14.0] - 2025-11-24

### 🚀 Features

- Pending facility queries in credit board (#3096)

### ⚙️ Miscellaneous Tasks

- Proper server shutdown (#3095)
## [0.13.1] - 2025-11-24

### 🚜 Refactor

- Remove deposit dependency (#3085)
## [0.13.0] - 2025-11-22

### 🚀 Features

- Credit honeycomb board (#3093)
## [0.12.3] - 2025-11-21

### ⚙️ Miscellaneous Tasks

- Add instrumentation for oathkeeper (#3089)
- Add new permission to create term templates (#3080)
- Missing field for oathkeeper otel (#3091)
## [0.12.2] - 2025-11-21

### 🚜 Refactor

- Move `close_monthly` to new `FiscalYear` entity (#2991)
## [0.12.1] - 2025-11-21

### ⚙️ Miscellaneous Tasks

- Add ledger transactions to sidebar (#3076)
## [0.12.0] - 2025-11-21

### 🚀 Features

- Use ephemeral events for price updates (#3084)

### ⚙️ Miscellaneous Tasks

- Skip posting transactions for zero-balance freeze/unfreeze (#3083)
## [0.11.2] - 2025-11-20

### ⚙️ Miscellaneous Tasks

- *(admin-panel)* Always show drop-down for create in details page (#3072)
- Improve pending facility tracing (#3074)
## [0.11.1] - 2025-11-20

### ⚙️ Miscellaneous Tasks

- Force scrollbar visibility on the Journal page (#3075)
## [0.11.0] - 2025-11-20

### 🚀 Features

- Dagster lana dw baby step one (#3055)

### ⚙️ Miscellaneous Tasks

- Source price from outbox  (#3050)
## [0.10.9] - 2025-11-20

### 🚜 Refactor

- *(deposit-account)* Make events expressive and isolate commands (#3062)

### ⚙️ Miscellaneous Tasks

- Bump flake (#3064)
- Remove outdated explanation in readme (#3073)
- Update job crate to 0.2.9 (#3071)
## [0.10.8] - 2025-11-19

### ⚙️ Miscellaneous Tasks

- Remove unused tables (#3066)
## [0.10.7] - 2025-11-19

### ⚙️ Miscellaneous Tasks

- Release lock on failure
- Journal ui improvments (#3054)
## [0.10.6] - 2025-11-18

### ⚙️ Miscellaneous Tasks

- Using display directly in macro (#3056)
- Add new board (#3017)
## [0.10.5] - 2025-11-18

### 🚜 Refactor

- Customer activity module (#3052)

### ⚙️ Miscellaneous Tasks

- Fmt dagster code with make (#3042)
## [0.10.4] - 2025-11-18

### 🐛 Bug Fixes

- Pnpm audit issues (#3053)

### 🧪 Testing

- Update customer bats and test account close (#3045)

### ⚙️ Miscellaneous Tasks

- Tracing improvements for proposal entity  (#3048)
- Restrict ledger activities for closed and frozen accounts (#3049)
## [0.10.3] - 2025-11-15

### ⚙️ Miscellaneous Tasks

- Bump versions with ARG, update readme (#3040)
- Schedule bitfinex el (#3029)
- *(deposit-accounts)* Move account to separate page and add close-account flow (#2982)
## [0.10.2] - 2025-11-14

### ⚙️ Miscellaneous Tasks

- Remove storybook addon-postcss (#3038)
## [0.10.1] - 2025-11-13

### ⚙️ Miscellaneous Tasks

- Fix releasing the release-lock
## [0.10.0] - 2025-11-13

### 🚀 Features

- Add ephemeral events in outbox (#3009)

### ⚙️ Miscellaneous Tasks

- Use release-lock to prohibit version bump overlap
## [0.9.2] - 2025-11-12

### ⚙️ Miscellaneous Tasks

- Add span for collateralization updated fn (#3027)
## [0.9.1] - 2025-11-12

### ⚙️ Miscellaneous Tasks

- Remove accountId from proposal create input (#3028)
## [0.9.0] - 2025-11-12

### 🚀 Features

- Create deposit account manually (#3019)
## [0.8.1] - 2025-11-11

### 🐛 Bug Fixes

- Naming of fn params (#3020)
- Correctly parsing headers (#3021)

### ⚙️ Miscellaneous Tasks

- Remove unnecessary refs to otel env var (#3022)
- Bump files, do some DRY (#3023)
## [0.8.0] - 2025-11-11

### 🚀 Features

- Add dlt-based extract and load for bitfinex api (#2981)
## [0.7.6] - 2025-11-11

### 🚜 Refactor

- Remove `chart_id` param from lib functions (#3018)
## [0.7.5] - 2025-11-11

### ⚙️ Miscellaneous Tasks

- Adapt sim bootstrap after introducing disbursal policies (#3016)
## [0.7.4] - 2025-11-11

### 🐛 Bug Fixes

- Add pending facility collateralization to history (#3014)
## [0.7.3] - 2025-11-10

### ⚙️ Miscellaneous Tasks

- [**breaking**] Update license to BSL (#3005)
## [0.7.2] - 2025-11-07

### 🐛 Bug Fixes

- Remove external lana-bank_default network from dagster compose (#3015)

### 🧪 Testing

- Simplify checks and fail properly (#3008)

### ⚙️ Miscellaneous Tasks

- Otel for dagster (#3011)

### ◀️ Revert

- "chore: remove redundante table creation (#2995)" (#3010)
## [0.7.1] - 2025-11-07

### ⚙️ Miscellaneous Tasks

- More boards for jobs (#2894)
## [0.7.0] - 2025-11-07

### 🚀 Features

- Unfreeze deposit account (#3000)
## [0.6.7] - 2025-11-06

### 🐛 Bug Fixes

- Use pending facility collateralization state event for activation job (#3001)

### ⚙️ Miscellaneous Tasks

- Minor cleanups (#3002)
## [0.6.6] - 2025-11-06

### 🐛 Bug Fixes

- Single Disbursal as default in sim-bootstrap (#3003)
## [0.6.5] - 2025-11-06

### ⚙️ Miscellaneous Tasks

- Remove redundante table creation (#2995)
- Attempt to fix version race issue
- Refactor loop on graphql layer (#2998)
## [0.6.4] - 2025-11-05

### ⚙️ Miscellaneous Tasks

- Update the job crate dependency to 0.2.7 (#2992)

### ◀️ Revert

- *(reporting)* Run full ETL on on-demand report generation (#2996)
## [0.6.3] - 2025-11-05

### ⚙️ Miscellaneous Tasks

- Add 2 more indexes for jobs (#2968)
## [0.6.2] - 2025-11-05

### ⚙️ Miscellaneous Tasks

- *(reporting)* Run full ETL on on-demand report generation (#2988)
## [0.6.1] - 2025-11-05

### ⚙️ Miscellaneous Tasks

- Remove healthcheck and adjust dependency (#2990)
## [0.6.0] - 2025-11-04

### 🚀 Features

- Tests now include some basic dagster checks (#2961)

### 🐛 Bug Fixes

- *(reporting)* Install dbt dependencies before running (#2987)

### ⚙️ Miscellaneous Tasks

- Auto customer-approval in bootstrap scenarios (#2985)
- Make dg pg healthcheck params more generous with time (#2986)
## [0.5.11] - 2025-11-04

### 🐛 Bug Fixes

- Single disbursement UI bugs (#2974)
## [0.5.9] - 2025-11-03

### 🐛 Bug Fixes

- Authz in missing places (#2973)
## [0.5.8] - 2025-11-03

### ⚙️ Miscellaneous Tasks

- *(bq-setup)* Holistics dataviewer did not depend on dataset (#2975)
## [0.5.7] - 2025-11-03

### ⚙️ Miscellaneous Tasks

- Dagster digest not tag (#2936)
- Split dockerfile in two (#2978)
## [0.5.6] - 2025-11-02

### ⚙️ Miscellaneous Tasks

- Update deps for new otel (#2972)
## [0.5.5] - 2025-11-02

### ⚙️ Miscellaneous Tasks

- More otel (#2971)
## [0.5.4] - 2025-11-02

### ⚙️ Miscellaneous Tasks

- Remove duplicate initialization (#2970)
## [0.5.3] - 2025-11-01

### ⚙️ Miscellaneous Tasks

- Remove legacy kratos files (#2967)
- Trace fine tuning (#2969)
## [0.5.2] - 2025-10-31

### 🐛 Bug Fixes

- Pick up whole repo, not just dagster (#2963)

### ⚙️ Miscellaneous Tasks

- Add polling for facility proposal (#2965)
## [0.5.1] - 2025-10-30

### ⚙️ Miscellaneous Tasks

- Proper context for where to build image (#2962)
## [0.5.0] - 2025-10-30

### 🚀 Features

- Local dagster env (#2954)
## [0.4.6] - 2025-10-30

### ⚙️ Miscellaneous Tasks

- Skip sumsub itest if unable to pull secrets for set environment vars (#2959)
## [0.4.5] - 2025-10-30

### ⚙️ Miscellaneous Tasks

- Add customer approval step in proposal entity (#2927)
## [0.4.4] - 2025-10-29

### 🐛 Bug Fixes

- `build-customer-portal-edge-image` failing - eliminate `shared-web` `tsconfig.json` extension (#2953)

### ⚙️ Miscellaneous Tasks

- Force shared-web tsconfig.json update (#2955)
- Instrumenting boostrap (#2948)
## [0.4.3] - 2025-10-28

### 🐛 Bug Fixes

- Matched shared-web peerDependencies to impacted customer-portal and admin-panel dep versions (#2952)
## [0.4.1] - 2025-10-28

### 🐛 Bug Fixes

- Fmt (#2949)

### ⚙️ Miscellaneous Tasks

- More trace updates
## [0.4.0] - 2025-10-28

### 🚀 Features

- Add support for single disbursement (#2850)
## [0.3.1006] - 2025-10-28

### 🚜 Refactor

- Use es-entity maybe_ fns (#2944)

### ⚙️ Miscellaneous Tasks

- Remove dead code related to browserstack (#2943)
## [0.3.1005] - 2025-10-28

### ⚙️ Miscellaneous Tasks

- Update deps with traces updates (#2942)
## [0.3.1004] - 2025-10-28

### ⚙️ Miscellaneous Tasks

- More trace updates (#2938)
## [0.3.1003] - 2025-10-27

### ⚙️ Miscellaneous Tasks

- Bump deps (#2937)
## [0.3.1002] - 2025-10-27

### 📚 Documentation

- Update cypress test instructions (#2928)
## [0.3.1001] - 2025-10-27

### ⚙️ Miscellaneous Tasks

- Update cala (#2934)
- Highlight root accounts in trial balance UI (#2895)
## [0.3.1000] - 2025-10-24

### ⚙️ Miscellaneous Tasks

- Remove wait-for-cachix in GitHub actions bats (#2933)
## [0.3.999] - 2025-10-24

### ⚙️ Miscellaneous Tasks

- More instrumentation (#2932)
## [0.3.998] - 2025-10-24

### ⚙️ Miscellaneous Tasks

- Re-trigger cache build post merge to main (#2923)
- Fix fly sha256
- Add curl to PATH
- Trigger cache on repo / version change
- Update es entity (#2930)
## [0.3.997] - 2025-10-23

### ⚙️ Miscellaneous Tasks

- *(reporting)* Data tests for nrp51 saldo_cuenta (#2913)
## [0.3.996] - 2025-10-23

### ⚙️ Miscellaneous Tasks

- Only build dagster on dagster/* changes (#2925)
- Fix dagster trigger
- *(reporting)* Data tests for nrp-41 reports (#2875)
## [0.3.995] - 2025-10-23

### ⚙️ Miscellaneous Tasks

- Formatting ci/flake.nix
- More deps update (#2924)
## [0.3.994] - 2025-10-23

### ⚙️ Miscellaneous Tasks

- Update otel (#2921)
## [0.3.993] - 2025-10-22

### ⚙️ Miscellaneous Tasks

- Update more deps (#2918)
## [0.3.992] - 2025-10-22

### ⚙️ Miscellaneous Tasks

- Fix release job
- Remove serial: true for build-release-main
- Auto-bump-patch via resource
- Deps updates (#2917)
## [0.3.991] - 2025-10-22

### ⚙️ Miscellaneous Tasks

- Remove synchronous read (#2911)
## [0.3.990] - 2025-10-22

### 🐛 Bug Fixes

- Ci/flake.nix fmt

### ⚙️ Miscellaneous Tasks

- Set-next-version task
- Use cocogitto to bump version
- Test dummy commit
- Add passed: [ set-next-version ]
## [0.3.989] - 2025-10-22

### 🐛 Bug Fixes

- Make e2e command (#2887)
## [0.3.988] - 2025-10-22

### ⚙️ Miscellaneous Tasks

- Also trigger when main gets tagged
- Fix resource_name
- Populate honeycomb into charts (#2908)
- Compare against .git/resource/head_sha
- Fix honeycomb boards (#2914)
## [0.3.987] - 2025-10-20

### ⚙️ Miscellaneous Tasks

- Use next-version in release
- Fix release
- Allow parallel access for access_token (#2897)
- Rename proposal -> pending in some places (#2885)
- Fetch_tags for release
- Resolve current alerts and add more otel info (#2896)
## [0.3.985] - 2025-10-20

### 🚀 Features

- Add dockerfile for dagster code location (#2899)

### ⚙️ Miscellaneous Tasks

- Nix-caching (#2893)
- Use galoymoney cache in gh actions
- Use galoymoney cache in release pipeline
- Wait-cachix-paths in flake (#2900)
## [0.3.984] - 2025-10-18

### ⚙️ Miscellaneous Tasks

- Flatten trial balance (#2874)
## [0.3.983] - 2025-10-17

### ⚙️ Miscellaneous Tasks

- Log part of the payload when there is an issue (#2832)
## [0.3.982] - 2025-10-17

### ⚙️ Miscellaneous Tasks

- Make release depend on meltano image + skip download for performance (#2888)
- Cache Lana deps (#2889)
- Instrumenting more sumsub (#2892)
## [0.3.981] - 2025-10-17

### 💼 Other

- Add nix-cache pipeline for release build (#2886)
## [0.3.980] - 2025-10-16

### ⚙️ Miscellaneous Tasks

- Slower cronjob for simtime (#2883)
## [0.3.979] - 2025-10-16

### 🐛 Bug Fixes

- Missing indefinite retry for outbox jobs (#2884)
## [0.3.978] - 2025-10-16

### 🐛 Bug Fixes

- Return Ok when facility not present for collateralization update (#2882)
## [0.3.977] - 2025-10-16

### ⚙️ Miscellaneous Tasks

- Local dev for honeycomb boards (#2857)
## [0.3.976] - 2025-10-16

### 🐛 Bug Fixes

- Duplicated credit facility - wip 503 (#2869)
## [0.3.975] - 2025-10-16

### 🐛 Bug Fixes

- Active credit facility report - wip 505 (#2870)
## [0.3.974] - 2025-10-16

### 💼 Other

- Allow version override via RELEASE_BUILD_VERSION env var (#2871)
## [0.3.973] - 2025-10-16

### ⚙️ Miscellaneous Tasks

- Remove redundant make command and refer correct command in flake (#2876)
## [0.3.972] - 2025-10-15

### ⚙️ Miscellaneous Tasks

- Remove unexposed config variable (#2868)
## [0.3.971] - 2025-10-15

### 🚀 Features

- Differentiate between pending facilities and proposals  (#2802)
## [0.3.970] - 2025-10-15

### 🐛 Bug Fixes

- Interest accrual cycle bats
## [0.3.969] - 2025-10-15

### ⚙️ Miscellaneous Tasks

- Conventional commit check (#2843)
## [0.3.968] - 2025-10-14

### ⚙️ Miscellaneous Tasks

- Bump cala to 0.12.1 - exposes balance_type on AccountBalance (#2864)
## [0.3.967] - 2025-10-14

### ⚙️ Miscellaneous Tasks

- Remove leftover instrument and make command (#2862)
## [0.3.966] - 2025-10-14

### ⚙️ Miscellaneous Tasks

- Change name to match method (#2860)
## [0.3.965] - 2025-10-14

### 🚜 Refactor

- Credit facility injection in disbursal (#2852)
## [0.3.964] - 2025-10-13

### ⚙️ Miscellaneous Tasks

- Another round of tracing (#2848)
## [0.3.963] - 2025-10-13

### ⚙️ Miscellaneous Tasks

- Bump job to include tracing (#2854)
## [0.3.962] - 2025-10-13

### 🚜 Refactor

- Improve activate use case (#2847)
## [0.3.960] - 2025-10-12

### ⚙️ Miscellaneous Tasks

- Use job crate (#2734)
## [0.3.959] - 2025-10-10

### ⚙️ Miscellaneous Tasks

- More traces (#2841)
## [0.3.958] - 2025-10-09

### 🧪 Testing

- Rename credit-facility.bats -> credit-facility-proposal.bats

### ⚙️ Miscellaneous Tasks

- Remove static-binary-image
- Correct nesting of on_failure hook
- Ignore Airflow errors (#2846)
## [0.3.957] - 2025-10-09

### 🧪 Testing

- Skip sumsub test if env is not setup
- Fix sumsub test

### ⚙️ Miscellaneous Tasks

- Nix flake check (#2742)
- Use --no-pretty in bats
- --no-pretty does not exist
- Attempt to add fonts to .#nextest
- Add libuuid to bats
- Wait for keycloak in bats
- Add production build (#2842)
## [0.3.955] - 2025-10-08

### ⚙️ Miscellaneous Tasks

- Remove create_credit_facility template (#2837)
- Do not override dataset for otel (#2830)
## [0.3.953] - 2025-10-08

### 🚜 Refactor

- *(credit)* Separate structuring fee into a new template (#2817)
## [0.3.951] - 2025-10-06

### ⚙️ Miscellaneous Tasks

- Wait for keycloak to be ready (#2833)
## [0.3.950] - 2025-10-04

### ⚙️ Miscellaneous Tasks

- "Fecha de emisión" to "Fecha de desembolso" (#2826)
- Improving sync fn (#2824)
## [0.3.949] - 2025-10-03

### ⚙️ Miscellaneous Tasks

- Report filters into shared component (#2827)
## [0.3.947] - 2025-10-02

### ⚙️ Miscellaneous Tasks

- Clean up of bats lana.yml (#2807)
## [0.3.946] - 2025-10-02

### 🐛 Bug Fixes

- Removing config for deposit config (#2811)
## [0.3.945] - 2025-10-02

### ⚙️ Miscellaneous Tasks

- Remove auto create deposit account config (#2820)
## [0.3.944] - 2025-10-02

### ⚙️ Miscellaneous Tasks

- Remove upgrade buffer config from yaml (#2819)
## [0.3.942] - 2025-10-01

### ⚙️ Miscellaneous Tasks

- Removing secret from yaml (#2821)
## [0.3.941] - 2025-10-01

### ⚙️ Miscellaneous Tasks

- Run dbt tests in github actions (#2780)
## [0.3.940] - 2025-10-01

### ⚙️ Miscellaneous Tasks

- Exclude trial balance children without code or balance (#2818)
## [0.3.939] - 2025-10-01

### 🐛 Bug Fixes

- Add missing facility activation event (#2816)
## [0.3.938] - 2025-09-30

### 🚀 Features

- Otel agent dev dataset tagging (#2810)
## [0.3.935] - 2025-09-30

### 🚀 Features

- Add monthly close (#2722)
## [0.3.934] - 2025-09-30

### ⚙️ Miscellaneous Tasks

- Remove unused fn test (#2812)
## [0.3.933] - 2025-09-30

### ⚙️ Miscellaneous Tasks

- Show full hierarchy in Trial balance (#2809)
## [0.3.932] - 2025-09-30

### ⚙️ Miscellaneous Tasks

- Refactor structuring_fee logic out of activate_credit_facility (#2803)
## [0.3.929] - 2025-09-29

### ⚙️ Miscellaneous Tasks

- Removing data pipeline from dependabot tests (#2804)
## [0.3.928] - 2025-09-29

### ⚙️ Miscellaneous Tasks

- Rename obligation installment (#2792)
## [0.3.926] - 2025-09-26

### ⚙️ Miscellaneous Tasks

- Can_have_manual_transactions returns bool (#2799)
## [0.3.925] - 2025-09-26

### ⚙️ Miscellaneous Tasks

- Link domain transaction entities->ledger transactions (#2791)
## [0.3.924] - 2025-09-26

### 🚀 Features

- Hardcode nrp28 report generation (#2797)

### ⚙️ Miscellaneous Tasks

- Run sqlfluff lint on changes to the data pipeline (#2775)
## [0.3.923] - 2025-09-25

### 🚜 Refactor

- Register children to ChartNode Entity (#2766)
## [0.3.922] - 2025-09-25

### 🚀 Features

- Credit facility proposal integration (#2795)
## [0.3.921] - 2025-09-24

### ⚙️ Miscellaneous Tasks

- Detail what parts of uif are pending (#2787)
## [0.3.920] - 2025-09-23

### ⚙️ Miscellaneous Tasks

- *(reporting)* Consistent names for meltano make targets (#2774)
## [0.3.919] - 2025-09-23

### ⚙️ Miscellaneous Tasks

- Drop outdated model (#2786)
- Add ledger account link for deposit account (#2783)
## [0.3.918] - 2025-09-23

### 🚀 Features

- Display credit-facility ledger accounts (#2777)
- Integrate proposals with facility (#2669)
## [0.3.917] - 2025-09-23

### 🐛 Bug Fixes

- Use collateral id from credit facility (#2781)
## [0.3.916] - 2025-09-22

### 🐛 Bug Fixes

- Use iptable for linux platforms (#2776)

### ⚙️ Miscellaneous Tasks

- Link disbursal entity to ledger txn and collateral entity to ledger account (#2726)
## [0.3.915] - 2025-09-20

### 🚀 Features

- Add transactions to uif (#2765)
- *(reporting)* Use public ids in reports (#2759)

### 🐛 Bug Fixes

- Pipeline chart of account sync with backend (#2770)

### ⚙️ Miscellaneous Tasks

- Data pipeline (#2748)
- Bump flake (#2769)
- Add iptables to flake.nix
## [0.3.914] - 2025-09-19

### ⚙️ Miscellaneous Tasks

- Include new_entities when checking for node in chart (#2767)
## [0.3.913] - 2025-09-19

### ⚙️ Miscellaneous Tasks

- *(admin-panel)* Use publicId for withdrawal and deposit (#2764)
## [0.3.912] - 2025-09-18

### ⚙️ Miscellaneous Tasks

- Link ledger view to entity (#2754)
- Add public_id for deposit and withdraw (#2762)
## [0.3.910] - 2025-09-17

### ⚙️ Miscellaneous Tasks

- Remove builder pattern from CoA configuration structs (#2670)
## [0.3.909] - 2025-09-17

### 🚜 Refactor

- Implement ChartNode entity (#2728)
## [0.3.908] - 2025-09-17

### 📚 Documentation

- *(onboarding)* Generalize command for customer-portal w/ docker locally on macos (#2760)
## [0.3.907] - 2025-09-17

### ⚙️ Miscellaneous Tasks

- Bump es-entity / cala
## [0.3.905] - 2025-09-16

### 🚀 Features

- Nrp 28 annex 1 liquidity reserve report (#2709)

### ⚙️ Miscellaneous Tasks

- Remove dbt posthook (#2755)
## [0.3.904] - 2025-09-16

### 🐛 Bug Fixes

- Pipeline 20250916 (#2753)
## [0.3.902] - 2025-09-15

### 🚀 Features

- Run dbt seed before dbt run (#2745)
- Add seed for address (#2744)

### 🚜 Refactor

- Move entity rollups to dev/ (#2751)

### ⚙️ Miscellaneous Tasks

- Add tracing to simulation scenarios (#2749)
## [0.3.901] - 2025-09-15

### ⚙️ Miscellaneous Tasks

- Bump es-entity / cala-ledger
## [0.3.899] - 2025-09-12

### 🚀 Features

- Placeholder for uif 07 report output (#2720)

### ⚙️ Miscellaneous Tasks

- Use lana-bank-ci cachix cache (#2743)
## [0.3.898] - 2025-09-11

### ⚙️ Miscellaneous Tasks

- Allow delete content on destroy dbt dataset (#2739)
## [0.3.897] - 2025-09-11

### 🚀 Features

- Split pg to bq job from dbt run job (#2738)

### ⚙️ Miscellaneous Tasks

- Bump cala
## [0.3.895] - 2025-09-11

### 🐛 Bug Fixes

- Pipeline json timestamp parsing (#2737)
## [0.3.894] - 2025-09-10

### ⚙️ Miscellaneous Tasks

- Rename InterestAccrualCycleAccountIds->InterestAccrualCycleLedgerAccountIds (#2733)
## [0.3.893] - 2025-09-10

### 🚀 Features

- Add tap sumsub api logging (#2732)
## [0.3.892] - 2025-09-10

### 🐛 Bug Fixes

- Pipeline 20250910 (#2731)
## [0.3.891] - 2025-09-09

### 🚀 Features

- Add duplicate term template (#2727)

### 🐛 Bug Fixes

- Update descriptions of active facilites and total disbursed (#2715)
- Maintain font size when menu items are active (#2716)
## [0.3.889] - 2025-09-08

### ⚙️ Miscellaneous Tasks

- Clarifying docs with small quirk on linux (#2723)
## [0.3.888] - 2025-09-08

### ⚙️ Miscellaneous Tasks

- Bump cala
## [0.3.887] - 2025-09-05

### 🚀 Features

- Capitalizing sentence works with spanish chars (#2719)

### ⚙️ Miscellaneous Tasks

- Link entities -> ledger transactions and expose in gql (#2717)
## [0.3.886] - 2025-09-05

### 🚀 Features

- Expand report file name REGEX with more spanish chars (#2718)

### ⚙️ Miscellaneous Tasks

- *(ui)* Update layer column position (#2713)
## [0.3.885] - 2025-09-04

### 🐛 Bug Fixes

- Use correct names in deposit account gql (#2710)
- Core deposit account events rollup renaming (#2708)
- Remove author typo and wrong header naming (#2712)

### ⚙️ Miscellaneous Tasks

- Expose ledger accounts from credit facility in gql (#2711)
- Expose credit facility entity in ledger accounts gql for cross-linking (#2707)
## [0.3.884] - 2025-09-04

### 🚜 Refactor

- Update customer-activity crate (#2690)
## [0.3.883] - 2025-09-04

### ⚙️ Miscellaneous Tasks

- Expose ledger accounts for deposit accounts in gql (#2694)
## [0.3.882] - 2025-09-03

### ⚙️ Miscellaneous Tasks

- Another pnpm update (#2704)
## [0.3.881] - 2025-09-03

### ⚙️ Miscellaneous Tasks

- Update pnpm (#2703)
## [0.3.880] - 2025-09-03

### ⚙️ Miscellaneous Tasks

- Add reference from LedgerTransaction -> Deposit (#2701)
## [0.3.879] - 2025-09-03

### 🐛 Bug Fixes

- Bump versions to fix security issues (#2702)
## [0.3.878] - 2025-09-03

### ⚙️ Miscellaneous Tasks

- Bump cala / es-entity (#2698)
## [0.3.877] - 2025-09-03

### 🚀 Features

- Add command to empty bq dataset (#2696)
## [0.3.876] - 2025-09-03

### ⚙️ Miscellaneous Tasks

- Expose LedgerAccountEntity to cross reference accounting -> core (#2697)
## [0.3.875] - 2025-09-03

### 🐛 Bug Fixes

- Add retries on concurrent modification in simulations (#2695)

### ⚙️ Miscellaneous Tasks

- Rename payment mutation and actions (#2680)
- *(reporting)* Add non-regulatory reports to frontend (#2683)
## [0.3.874] - 2025-09-01

### 🐛 Bug Fixes

- Use separate global and entity id for ledger account in gql (#2691)

### ⚙️ Miscellaneous Tasks

- Order children of a ledger account by account code (#2663)
## [0.3.873] - 2025-09-01

### 🚀 Features

- Display account activity and update KYC status (#2687)
## [0.3.872] - 2025-09-01

### 🐛 Bug Fixes

- Tracing with updated libs

### ⚙️ Miscellaneous Tasks

- Show layer in journal (#2684)
- Update custody ui (#2688)
- Remove sumsub link from list page (#2685)

### ◀️ Revert

- "chore: Revert upgrade of otel libraries to fix tracing (#2686)"
## [0.3.871] - 2025-09-01

### 🚜 Refactor

- Pass in the props instead of fetching again using publicId (#2681)

### ⚙️ Miscellaneous Tasks

- Revert upgrade of otel libraries to fix tracing (#2686)
## [0.3.870] - 2025-08-29

### 🐛 Bug Fixes

- Credit proposal approve job overriding correct credit approve job (#2679)
## [0.3.869] - 2025-08-29

### ⚙️ Miscellaneous Tasks

- Introduce new permission_set for creating payments with custom effective dates (#2636)
## [0.3.868] - 2025-08-29

### ⚙️ Miscellaneous Tasks

- *(translation)* Additional translation tweaks (#2674)
- Tracing keycloak (#2592)
## [0.3.867] - 2025-08-29

### 🚜 Refactor

- Remove audit_info from domain events (#2667)
## [0.3.866] - 2025-08-29

### 🐛 Bug Fixes

- Full refrsh flags (#2677)
## [0.3.865] - 2025-08-29

### 🐛 Bug Fixes

- Timestamp in gcs blob path causes signature issues (#2675)
## [0.3.864] - 2025-08-29

### 🐛 Bug Fixes

- Pipeline 20250829 (#2676)
## [0.3.862] - 2025-08-28

### 🐛 Bug Fixes

- Pipeline 20250828 (#2671)
## [0.3.861] - 2025-08-28

### ⚙️ Miscellaneous Tasks

- Fix minimum date error in activity_check (#2668)
## [0.3.859] - 2025-08-27

### 🐛 Bug Fixes

- Remove large spec column (#2665)
## [0.3.858] - 2025-08-27

### ⚙️ Miscellaneous Tasks

- Fix account code of frozen non-domiciled individuals (#2664)
## [0.3.857] - 2025-08-27

### 🐛 Bug Fixes

- Pipeline 20250827 (#2662)
## [0.3.856] - 2025-08-27

### ⚙️ Miscellaneous Tasks

- Fix names of frozen accounts in GQL (#2661)
## [0.3.854] - 2025-08-27

### ⚙️ Miscellaneous Tasks

- Add frozen accounts to admin server's module config (#2653)
## [0.3.853] - 2025-08-26

### ⚙️ Miscellaneous Tasks

- Add ledger transaction link to account journal entries (#2648)
- Example should exists (#2657)
- Moving trial balance around (#2656)
- *(reporting)* Only extract and load required tables from pg (#2654)
## [0.3.852] - 2025-08-26

### 🚜 Refactor

- Use collateralization ratio (#2650)
## [0.3.851] - 2025-08-25

### 🚀 Features

- Credit facility proposal (#2635)

### 🐛 Bug Fixes

- Data pipeline missing sources (#2651)
## [0.3.850] - 2025-08-25

### ⚙️ Miscellaneous Tasks

- Integrate sqlx migration of public id sequence into setup file (#2646)
- *(reporting)* Reduce dbt frequency (#2647)
## [0.3.849] - 2025-08-25

### 🚜 Refactor

- Introduce explicit `CollateralizationRatio` type (#2641)
## [0.3.848] - 2025-08-25

### 🐛 Bug Fixes

- *(admin-panel)* Details card alignment and dev warnings (#2644)
## [0.3.847] - 2025-08-22

### ⚙️ Miscellaneous Tasks

- *(translation)* Improved translation for accounting terms (#2642)
- Removing hard coding of type for notification (#2632)
## [0.3.846] - 2025-08-22

### 🐛 Bug Fixes

- Handle errors for policy_init (#2643)
## [0.3.845] - 2025-08-22

### 🚀 Features

- Add freezing of deposit accounts (#2585)

### ⚙️ Miscellaneous Tasks

- Start public_id at 1000 (#2638)
- Change due_at dates to EffectiveDate (#2623)
## [0.3.844] - 2025-08-22

### 🐛 Bug Fixes

- Add missing casbin policy updates when creating roles/updating policies (#2611)
## [0.3.843] - 2025-08-22

### 📚 Documentation

- Lana mdbook (#2606)

### ⚙️ Miscellaneous Tasks

- Display manaul custodian on facility page (#2633)
## [0.3.842] - 2025-08-22

### 📚 Documentation

- Add note about Brave local admin login (#2631)
## [0.3.841] - 2025-08-21

### 🚀 Features

- *(ui)* Idle session guard (#2613)
## [0.3.840] - 2025-08-21

### ⚙️ Miscellaneous Tasks

- Remove clippy warning (#2621)
- Remove stale .gql file from bats (#2628)
## [0.3.839] - 2025-08-21

### ⚙️ Miscellaneous Tasks

- Report job updates (#2527)
## [0.3.838] - 2025-08-21

### 🐛 Bug Fixes

- Generate-es-reports run command (#2627)

### ⚙️ Miscellaneous Tasks

- Change Obligation/Disbursal lifecycle dates to EffectiveDate (#2622)
## [0.3.837] - 2025-08-21

### ⚙️ Miscellaneous Tasks

- Remove redundant steps from authenticationFlows (#2625)
## [0.3.836] - 2025-08-21

### 🐛 Bug Fixes

- Links to ledger accounts broken (#2624)
## [0.3.835] - 2025-08-21

### 🐛 Bug Fixes

- Payment allocation to obligation installment (#2616)
## [0.3.834] - 2025-08-21

### ⚙️ Miscellaneous Tasks

- Use start_of_day for obligation dates and expected status (#2620)
## [0.3.832] - 2025-08-20

### 🚀 Features

- *(customer-sync)* Add customer activity check job (#2562)
## [0.3.831] - 2025-08-20

### ⚙️ Miscellaneous Tasks

- *(ui)* Adding custodian details (#2617)
- *(docs)* Add authorization doc (#2609)
## [0.3.830] - 2025-08-20

### 🚀 Features

- Add more details about collateral custody (#2610)

### ⚙️ Miscellaneous Tasks

- Minor translation tweaks (#2612)
## [0.3.829] - 2025-08-18

### ⚙️ Miscellaneous Tasks

- Reduce visibility of some struct (#2583)
- Remove redundant _accrual_cycle from NewAccrualPeriods (#2547)
- Update concourse (#2429)
## [0.3.828] - 2025-08-18

### 🚜 Refactor

- Add matured event (#2594)
## [0.3.827] - 2025-08-18

### ⚙️ Miscellaneous Tasks

- Use From for error mapping (#2602)
- Add enabled flag to report config (#2603)
## [0.3.826] - 2025-08-18

### ⚙️ Miscellaneous Tasks

- Pass env variables from server (#2600)
- Cypress config changes for memory-management (#2595)
- Remove root_folder from gcs (#2604)
## [0.3.825] - 2025-08-14

### 🐛 Bug Fixes

- Get all scheduled runs (#2586)
## [0.3.823] - 2025-08-14

### ⚙️ Miscellaneous Tasks

- Improve code structure & entity tests (#2576)
## [0.3.822] - 2025-08-14

### ⚙️ Miscellaneous Tasks

- Remove redundant packages from admin-panel (#2596)
## [0.3.821] - 2025-08-14

### 🚜 Refactor

- Clarify type of status of Customer and Deposit Account  (#2584)
## [0.3.820] - 2025-08-13

### ⚙️ Miscellaneous Tasks

- Migrate to keycloak (#2535)
## [0.3.819] - 2025-08-13

### ⚙️ Miscellaneous Tasks

- Reduce tracing level of job.poll_and_dispatch (#2587)
## [0.3.818] - 2025-08-13

### ⚙️ Miscellaneous Tasks

- Refactor generate-es-reports (#2569)
## [0.3.817] - 2025-08-13

### ⚙️ Miscellaneous Tasks

- Check custodian config before storing (#2573)
## [0.3.816] - 2025-08-12

### ⚙️ Miscellaneous Tasks

- Remove unused import (#2579)
## [0.3.814] - 2025-08-12

### 🐛 Bug Fixes

- Current_cvl checks total_outstanding instead of disbursed (#2522)
## [0.3.812] - 2025-08-12

### 🚀 Features

- Link cf history with ledger transactions (#2302)

### 🚜 Refactor

- Specify path only in <root>/Cargo.toml (#2567)

### ⚙️ Miscellaneous Tasks

- Bump flake + clippy 1.89 (#2566)
- Bump es entity (#2572)
## [0.3.811] - 2025-08-08

### 🚀 Features

- Adjust reg report xml to schemas (#2561)
## [0.3.810] - 2025-08-08

### 🐛 Bug Fixes

- Lint

### ⚙️ Miscellaneous Tasks

- Make manual custody explicit (#2560)
## [0.3.809] - 2025-08-07

### ⚙️ Miscellaneous Tasks

- Showing what default values are for lana.yml (#2541)
## [0.3.808] - 2025-08-06

### ⚙️ Miscellaneous Tasks

- Use standard graphql object (#2556)
## [0.3.807] - 2025-08-06

### ⚙️ Miscellaneous Tasks

- Replacing subjet to user on graphql layer (#2552)
## [0.3.805] - 2025-08-06

### 🚜 Refactor

- Webhook in its own folder (#2511)
## [0.3.804] - 2025-08-06

### 🚀 Features

- Add xsd validation warning (#2540)

### 🚜 Refactor

- Another attempt at simplify authz (#2533)

### ⚙️ Miscellaneous Tasks

- Rename facility is_approval_allowed to is_activation_allowed (#2539)
- Report dated ordering (#2545)
- Improve dag (#2550)
## [0.3.803] - 2025-08-05

### ⚙️ Miscellaneous Tasks

- Update rollup schema (#2546)
## [0.3.802] - 2025-08-05

### 🚀 Features

- Declarative report file generation (#2538)
## [0.3.801] - 2025-08-05

### 🚜 Refactor

- Remove allocations from payments (#2536)
## [0.3.800] - 2025-08-05

### ⚙️ Miscellaneous Tasks

- Revert back some of the code from #2357 (#2544)
## [0.3.799] - 2025-08-04

### 🚀 Features

- Luis non-reg reports (#2454)
## [0.3.798] - 2025-08-04

### 🚀 Features

- Update rollups for running sum events (#2493)
## [0.3.797] - 2025-08-04

### 🚀 Features

- Use logger in es generator (#2526)
## [0.3.796] - 2025-08-04

### ⚙️ Miscellaneous Tasks

- Show max allowed digits on account creation (#2520)
- Make it possible to create customer on staging (#2532)
- Move price test to price crate (#2534)
- Publish collateral update when updated via custodian sync (#2537)
## [0.3.795] - 2025-08-01

### 🐛 Bug Fixes

- Add CONTRACT_CREATION enum (#2528)
## [0.3.794] - 2025-08-01

### 🚀 Features

- Report generation configurable output (#2518)

### 🐛 Bug Fixes

- Wallet details style (#2519)
## [0.3.793] - 2025-07-31

### 🧪 Testing

- See if this reduce flackyness (#2515)

### ⚙️ Miscellaneous Tasks

- Moving contract creation to its own crate (#2484)
- Removing unused code (#2513)
## [0.3.792] - 2025-07-31

### 🚜 Refactor

- Embed status in withdrawal events (#2517)

### ⚙️ Miscellaneous Tasks

- Add airflow compose to clean-deps (#2516)
## [0.3.791] - 2025-07-31

### 🚜 Refactor

- Embed status in deposit events (#2508)

### ⚙️ Miscellaneous Tasks

- Delete unused file (#2506)
- Add airflow to reset-deps workflow (#2504)
- Remove unused property from lana.yml (#2507)
- Simplifying workspace Cargo.toml (#2491)
- Small nits following applicant PR refactoring (#2488)
- Misc formatting (#2489)
## [0.3.790] - 2025-07-30

### ⚙️ Miscellaneous Tasks

- Include traces for balances repo in cala (#2497)
## [0.3.789] - 2025-07-30

### 🚜 Refactor

- Split add_node into add_root_node and add_child_node mutations (#2490)
## [0.3.788] - 2025-07-30

### ⚙️ Miscellaneous Tasks

- Improve meltano readme (#2494)
## [0.3.787] - 2025-07-30

### 🐛 Bug Fixes

- Remove / recreate bad rollup migrations

### 🚜 Refactor

- Make role mandatory (#2404)
## [0.3.786] - 2025-07-30

### 🚜 Refactor

- Insert in rollups (#2505)
## [0.3.785] - 2025-07-29

### ⚙️ Miscellaneous Tasks

- Is dev_disable still used (#2502)
## [0.3.784] - 2025-07-29

### 🐛 Bug Fixes

- Swap pg image to same one as app containers (#2499)
## [0.3.783] - 2025-07-29

### 🐛 Bug Fixes

- Airflow reports plugin should handle trailing slash (#2496)

### ⚙️ Miscellaneous Tasks

- Trim down meltano.nix (#2417)
## [0.3.782] - 2025-07-29

### 🚀 Features

- Core/report - connect runtime to airflow (#2357)

### ⚙️ Miscellaneous Tasks

- Update public id ui (#2495)
## [0.3.781] - 2025-07-29

### ⚙️ Miscellaneous Tasks

- Expose ledger BalanceType for ledger account (#2479)
## [0.3.780] - 2025-07-28

### ⚙️ Miscellaneous Tasks

- Moving applicant to its own crate (#2455)
## [0.3.779] - 2025-07-28

### ⚙️ Miscellaneous Tasks

- Make output less verbose + ci tool to detect DAG violation (#2477)
## [0.3.778] - 2025-07-28

### ⚙️ Miscellaneous Tasks

- Swap parts in webhook URL (#2487)
## [0.3.777] - 2025-07-28

### ⚙️ Miscellaneous Tasks

- Remove outdated version (#2483)
- Remove commented code from repository (#2435)
- Less verbose job instrumentation
## [0.3.776] - 2025-07-28

### ⚙️ Miscellaneous Tasks

- Allow to pass multiline secret key for Komainu (#2486)
## [0.3.775] - 2025-07-28

### ⚙️ Miscellaneous Tasks

- Use balance update time provided by custodian (#2485)
## [0.3.774] - 2025-07-27

### ⚙️ Miscellaneous Tasks

- Remove more commented code (#2482)
## [0.3.773] - 2025-07-27

### ⚙️ Miscellaneous Tasks

- Remove build timestamp (#2481)
## [0.3.772] - 2025-07-27

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.772-dev [ci skip]
- Remove unused code2 (#2480)
## [0.3.771] - 2025-07-27

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.771-dev [ci skip]
## [0.3.770] - 2025-07-26

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.769-dev [ci skip]
- Move to 2024 edition (#2478)
- Nits for the toml (#2475)
## [0.3.769] - 2025-07-25

### ⚙️ Miscellaneous Tasks

- Remove es-entity dependency for authz (#2470)
## [0.3.768] - 2025-07-25

### 🚜 Refactor

- *(graphql)* Add resolver to fetch latest accounting CSV (#2456)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.768-dev [ci skip]
- Remove dead code (#2469)
## [0.3.767] - 2025-07-25

### 🚜 Refactor

- Add status updated event type for deposit events (#2466)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.767-dev [ci skip]
## [0.3.766] - 2025-07-25

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.766-dev [ci skip]
- Change hard-coded external wallet ID (#2471)
## [0.3.765] - 2025-07-24

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.764-dev [ci skip]
## [0.3.764] - 2025-07-24

### 🚀 Features

- Allow to select BitGo as custody provider (#2443)
## [0.3.763] - 2025-07-24

### 🚀 Features

- Revert withdraw and deposit (#2462)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.763-dev [ci skip]
- Fix unwrap (#2465)
## [0.3.762] - 2025-07-23

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.762-dev [ci skip]
## [0.3.761] - 2025-07-23

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.761-dev [ci skip]
- Upgrading markdown2pdf (#2460)
## [0.3.760] - 2025-07-23

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.760-dev [ci skip]
## [0.3.759] - 2025-07-23

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.759-dev [ci skip]
- Second try for fonts embedding (#2461)
## [0.3.758] - 2025-07-22

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.758-dev [ci skip]
- Adding font to Docker for markdown2pdf (#2421)
## [0.3.757] - 2025-07-22

### 🐛 Bug Fixes

- Swap pg image to official one (#2457)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.757-dev [ci skip]
## [0.3.756] - 2025-07-22

### 🚀 Features

- Revert deposit (#2442)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.756-dev [ci skip]
## [0.3.755] - 2025-07-22

### 🐛 Bug Fixes

- Some job tracing improvements (#2453)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.755-dev [ci skip]
- Increate default_job_lost_interval
## [0.3.754] - 2025-07-22

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.754-dev [ci skip]
- Bump form-data (#2452)
## [0.3.753] - 2025-07-21

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.753-dev [ci skip]
- More consistent Cargo.toml (#2390)
## [0.3.752] - 2025-07-21

### 🚀 Features

- Ledger account search (#2444)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.752-dev [ci skip]
- Add tracing to panics (#2448)
## [0.3.751] - 2025-07-21

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.751-dev [ci skip]
- Update Komainu API base URL (#2445)
## [0.3.750] - 2025-07-21

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.750-dev [ci skip]
- Upgrade eslint-config-prettier (#2441)
## [0.3.749] - 2025-07-20

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.749-dev [ci skip]
- Adding otel to pdf creation jobs (#2439)
## [0.3.748] - 2025-07-20

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.747-dev [ci skip]
- Consistent use of dash (#2437)
- Implementing loop for get email (#2438)
## [0.3.747] - 2025-07-19

### 🚜 Refactor

- Add graphql feature to core/money (#2436)
## [0.3.746] - 2025-07-19

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.746-dev [ci skip]
- Improvment for path (#2434)
## [0.3.745] - 2025-07-19

### 🚀 Features

- Pdf contract creation (#2197)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.745-dev [ci skip]
## [0.3.744] - 2025-07-19

### 🧪 Testing

- In bats, wait for collateral update via custodian wallet sync (#2433)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.743-dev [ci skip]
## [0.3.742] - 2025-07-18

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.742-dev [ci skip]
## [0.3.741] - 2025-07-18

### 🚀 Features

- Revert withdrawal (#2399)

### 🚜 Refactor

- *(ui)* Use public id queries (#2424)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.741-dev [ci skip]
- Repayment plan zero accruals (#2398)
## [0.3.740] - 2025-07-18

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.740-dev [ci skip]
- Turn custodian webhooks into collateral updates (#2356)
## [0.3.739] - 2025-07-17

### 🐛 Bug Fixes

- Admin-panel warnings (#2413)
- Add rendering/test-output folder to .gitignore (#2425)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.739-dev [ci skip]
## [0.3.738] - 2025-07-17

### 🚜 Refactor

- Improve job crate (performance / tracing) (#2389)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.738-dev [ci skip]
## [0.3.737] - 2025-07-16

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.737-dev [ci skip]
- Upgrade to last markdown2pdf (#2419)
- Adding .devcontainer (#2187)
## [0.3.736] - 2025-07-16

### 🚀 Features

- Adding lib pdf rendering (#2406)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.736-dev [ci skip]
- Attempt at fixing concourse (#2414)
- Document updates (#2405)
- Ensuring protoc is available globally (#2416)
- Remove some warnings (#2415)
- New attempt at fixing protoc path (#2418)
- Fix fmt (#2420)
## [0.3.735] - 2025-07-16

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.735-dev [ci skip]
## [0.3.734] - 2025-07-15

### 🐛 Bug Fixes

- Customer portal warnings (#2407)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.734-dev [ci skip]
- Remove handleCustomerSearchFormSubmit (#2410)
- Other fix for the retry on the job lib (#2411)
## [0.3.733] - 2025-07-15

### 💼 Other

- Disable checks for all python311 packages (#2400)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.732-dev [ci skip]
## [0.3.732] - 2025-07-15

### 🧪 Testing

- Sumsub tests more complete (#2374)
## [0.3.731] - 2025-07-15

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.731-dev [ci skip]
## [0.3.730] - 2025-07-15

### 🚀 Features

- Add customer, credit facility, and disbursal queries by public ID (#2403)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.730-dev [ci skip]
## [0.3.729] - 2025-07-14

### ⚙️ Miscellaneous Tasks

- Fix non passing test for github actions (#2397)
- *(dev)* Set version to 0.3.729-dev [ci skip]
## [0.3.728] - 2025-07-14

### 🚀 Features

- Custodians UI (#2382)
- Expose build artefacts (#2385)
- Add public id for cf and disbursal (#2395)

### 🐛 Bug Fixes

- Meltano nix build (#2373)

### 💼 Other

- Credit facility report to use rollups (#2355)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.728-dev [ci skip]
- Rename CustodianEncryption -> Encryption (#2379)
- Public id (#2367)
- Moving script out of root (#2372)
- Add create_node function that validates parent (#2342)
- Add public id to DepositAccount (#2384)
## [0.3.727] - 2025-07-10

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.727-dev [ci skip]
- Validate to ensure no zero amount obligations (#2344)
## [0.3.726] - 2025-07-10

### 🐛 Bug Fixes

- Execution state usage patterns (#2364)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.726-dev [ci skip]
- Update nix task runner image (#2377)
## [0.3.725] - 2025-07-09

### 🚀 Features

- Add new accounts in Chart of Accounts (#2368)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.725-dev [ci skip]
- Use lana-dev-gcp-creds in concourse (#2360)
- This env var is no longer needed (#2370)
- Remove unused (#2371)
## [0.3.724] - 2025-07-09

### 🐛 Bug Fixes

- Fmt for meltano.nix (#2366)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.724-dev [ci skip]
## [0.3.723] - 2025-07-09

### 🐛 Bug Fixes

- *(dev)* Python3.11 fasterners skip interprocess_lock test in flake.nix (#2365)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.723-dev [ci skip]
## [0.3.722] - 2025-07-08

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.721-dev [ci skip]
- Refactor start-cypress-stack.sh to align with GitHub Actions (#2358)
## [0.3.721] - 2025-07-08

### ◀️ Revert

- Revert "chore: expose job in GQL (#2297)" (#2363)
## [0.3.720] - 2025-07-08

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.720-dev [ci skip]
## [0.3.719] - 2025-07-08

### 🚀 Features

- Airflow scheduler and webserver on docker (#2323)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.718-dev [ci skip]
- Simplify sumsub test to not depend on apps (#2317)
- Improve container engine setup and socket handling in flake.nix (#2353)
- Fix flake.nix formatting (#2362)
## [0.3.718] - 2025-07-08

### 🚀 Features

- Liquidation history entry (#2350)
## [0.3.717] - 2025-07-08

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.717-dev [ci skip]
- Fixing some label and improving config struct (#2345)
## [0.3.716] - 2025-07-08

### 🚜 Refactor

- Less duplication between docker and tilt (#2315)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.716-dev [ci skip]
- Iteration on podman configuration (#2351)
## [0.3.715] - 2025-07-07

### 🚀 Features

- Expose webhook URL for notifications from custodians (#2326)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.715-dev [ci skip]
## [0.3.714] - 2025-07-05

### 🚀 Features

- *(data)* Reports api (#2324)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.714-dev [ci skip]
## [0.3.713] - 2025-07-04

### ⚙️ Miscellaneous Tasks

- Fix fmt issues (#2337)
- *(dev)* Set version to 0.3.713-dev [ci skip]
- Cargo fmt (#2340)
## [0.3.712] - 2025-07-04

### 🚀 Features

- Add 'chartOfAccountsAddNode' mutation (#2288)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.712-dev [ci skip]
## [0.3.711] - 2025-07-04

### 💼 Other

- Report to use rollups (#2319)

### 🚜 Refactor

- Return `ChartOfAccounts` in csv import chart payload (#2333)

### 🧪 Testing

- Accounting csv export (#2329)
- Mock custodian (#2312)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.711-dev [ci skip]
- Adding credit facility link to email (#2320)
- Liquidation in bootstrap (#2309)
- Remove direct dependencies on core-* from admin-server (#2331)
- Bump flake (#2334)
- Bump deps (#2335)
## [0.3.710] - 2025-07-02

### 🚀 Features

- *(reporting)* Integrated report validations (#2307)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.710-dev [ci skip]
- Remove redundant setup-db step (#2328)
## [0.3.709] - 2025-07-02

### 🚜 Refactor

- Add outstanding_payable fields for disbursed and interest (#2308)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.709-dev [ci skip]
## [0.3.708] - 2025-07-02

### ⚙️ Miscellaneous Tasks

- Consistent button width on loading (#2314)
- *(dev)* Set version to 0.3.708-dev [ci skip]
## [0.3.707] - 2025-07-01

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.707-dev [ci skip]
## [0.3.706] - 2025-06-30

### 🐛 Bug Fixes

- Non deterministic test + refactor (#2311)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.706-dev [ci skip]
## [0.3.705] - 2025-06-30

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.705-dev [ci skip]
- Add custody as dep to credit (#2137)
## [0.3.704] - 2025-06-27

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.704-dev [ci skip]
## [0.3.703] - 2025-06-26

### 🚀 Features

- *(reporting)* Serialize reports differently depending on norm requirements (#2284)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.703-dev [ci skip]
## [0.3.702] - 2025-06-26

### 🚀 Features

- Constrain manual transactions to chart leaf accounts (#2298)

### 🐛 Bug Fixes

- Crash in the job library (#2299)
- Currency sign in account csv (#2303)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.701-dev [ci skip]
## [0.3.701] - 2025-06-26

### 🐛 Bug Fixes

- Cala -> lana in instrument name (#2300)
## [0.3.700] - 2025-06-26

### 🚀 Features

- Intermediate rollups (#2280)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.699-dev [ci skip]
- Expose job in GQL (#2297)
## [0.3.699] - 2025-06-25

### ⚙️ Miscellaneous Tasks

- Update localhost URLs for consistency (#2263)
## [0.3.698] - 2025-06-25

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.697-dev [ci skip]
- Remove hard coded not useful "system" audit log (#2296)
## [0.3.696] - 2025-06-25

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.696-dev [ci skip]
- Make create_in_op instead of create (#2293)
## [0.3.695] - 2025-06-25

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.695-dev [ci skip]
- *(reporting)* Better meltano sumsub config injection (#2291)
## [0.3.694] - 2025-06-25

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.693-dev [ci skip]
- Add overdue and liquidation duration to seed (#2283)
## [0.3.692] - 2025-06-25

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.691-dev [ci skip]
## [0.3.691] - 2025-06-24

### ⚙️ Miscellaneous Tasks

- Add liquidation to module config (#2245)
## [0.3.690] - 2025-06-24

### 🚀 Features

- Effective date in execute-manual-transaction (#2281)

### 🐛 Bug Fixes

- Apps build in ci (#2287)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.688-dev [ci skip]
- Remove accounting_csv entry (#2286)
## [0.3.687] - 2025-06-24

### 🚜 Refactor

- Use document-storage in account-csv (#2279)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.686-dev [ci skip]
## [0.3.686] - 2025-06-24

### 🚜 Refactor

- Lazy load storage client (#2278)
## [0.3.685] - 2025-06-24

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.685-dev [ci skip]
## [0.3.684] - 2025-06-23

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.684-dev [ci skip]
- Reduce the number of env variables (#2272)
## [0.3.683] - 2025-06-23

### 🚀 Features

- *(reporting)* PoC xml report generation in meltano (#2228)

### 🚜 Refactor

- Rename tx_id -> ledger_tx_id (#2268)
- -> Init as prefix for JobInitializers (#2274)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.681-dev [ci skip]
- Remove advisory (#2256)
- Fix some test and remove unused yml (#2259)
- Just one lana (#2269)
- Consistent naming for job initializers (#2273)
## [0.3.682] - 2025-06-23

### ⚙️ Miscellaneous Tasks

- Integrate liquidation ledger template in use-cases (#2239)
## [0.3.681] - 2025-06-23

### 🐛 Bug Fixes

- Meltano commands run without aliasing (#2266)
## [0.3.680] - 2025-06-23

### 🚀 Features

- Add 'reserve for liquidation' ledger template (#2223)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.679-dev [ci skip]
## [0.3.679] - 2025-06-23

### ⚙️ Miscellaneous Tasks

- In tests, read 'PGHOST' env var instead of 'PG_HOST' (#2265)
## [0.3.678] - 2025-06-23

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.678-dev [ci skip]
- Only recreate when changed (#2257)
## [0.3.677] - 2025-06-23

### 🚜 Refactor

- Default to local storage (#2264)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.677-dev [ci skip]
## [0.3.676] - 2025-06-22

### 🐛 Bug Fixes

- Accounting csv (#2262)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.675-dev [ci skip]
## [0.3.675] - 2025-06-22

### 🚀 Features

- Extract storage client (#2251)
## [0.3.674] - 2025-06-22

### 🐛 Bug Fixes

- Remove redundant query (#2261)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.673-dev [ci skip]
## [0.3.673] - 2025-06-21

### ⚙️ Miscellaneous Tasks

- Removing update-schemas-cargo from make code (#2255)
## [0.3.672] - 2025-06-21

### 🚀 Features

- *(ui)* Credit facility tweaks (#2234)
- Ncf-01 reg reports 1, 2, 3 of 4 (#2231)

### 🐛 Bug Fixes

- Fmt
- Update-schemas in nix
- Fmt
- Clipppy

### 🚜 Refactor

- DocumentType with Cow (#2246)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.672-dev [ci skip]
- Fix fmt (#2240)
- Bring back -cargo rules for now (#2243)
- Bump schemars 2 (#2244)
- Event rollup table generation (#2248)
- Use cargo for check-code in ci
- Remove entity-rollups templates from nix build
- Fix check-code (#2254)
## [0.3.671] - 2025-06-19

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.671-dev [ci skip]
## [0.3.670] - 2025-06-19

### 🚜 Refactor

- Proxy document storage via customer (#2232)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.669-dev [ci skip]
## [0.3.669] - 2025-06-18

### ⚙️ Miscellaneous Tasks

- Fix lint
## [0.3.668] - 2025-06-18

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.667-dev [ci skip]
- Add liquidation date to obligation (#2207)
## [0.3.667] - 2025-06-18

### 💼 Other

- Chart of account codification (#2230)
## [0.3.666] - 2025-06-18

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.666-dev [ci skip]
- Document-storage crate boilerplate (#2220)
## [0.3.665] - 2025-06-17

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.665-dev [ci skip]
- New test flake.nix (#2229)
## [0.3.664] - 2025-06-17

### 🚀 Features

- Add liquidation process entity (#2145)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.664-dev [ci skip]
## [0.3.663] - 2025-06-17

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.662-dev [ci skip]
## [0.3.662] - 2025-06-17

### 🚜 Refactor

- Rename credit job types (#2206)
## [0.3.661] - 2025-06-17

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.661-dev [ci skip]
## [0.3.660] - 2025-06-17

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.660-dev [ci skip]
## [0.3.659] - 2025-06-17

### 🐛 Bug Fixes

- Handle zero threshold (#2208)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.658-dev [ci skip]
## [0.3.658] - 2025-06-17

### ⚙️ Miscellaneous Tasks

- Removing env var that don't need to change (#2216)
- Add liquidation date to disbursal entity (#2211)
## [0.3.657] - 2025-06-17

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.657-dev [ci skip]
- Unused event (#2215)
## [0.3.656] - 2025-06-17

### 🐛 Bug Fixes

- Full refresh incremental tables (#2218)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.656-dev [ci skip]
- Switch lana bank cache name (#2205)
## [0.3.655] - 2025-06-16

### 🚀 Features

- *(admin-panel)* Show full timestamp in audit log (#2200)

### 💼 Other

- Add effective date in partial payment (#2202)

### 🚜 Refactor

- Approval threshold not greater than n_committee_members (#2198)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.655-dev [ci skip]
- Rename system approve -> auto approve (#2199)
- Output event schemas (#2185)
- Moving to rust 2024 (#2203)
- Some bats script cleanup (#2139)
## [0.3.654] - 2025-06-12

### 🚀 Features

- Custodian config encryption (#2157)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.654-dev [ci skip]
## [0.3.653] - 2025-06-12

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.652-dev [ci skip]
## [0.3.652] - 2025-06-12

### ⚙️ Miscellaneous Tasks

- Add mailcrab (#2189)
## [0.3.651] - 2025-06-12

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.649-dev [ci skip]
- Deactivating tests on python (#2188)
## [0.3.650] - 2025-06-12

### 🚀 Features

- Reg report use chart of accounts (#2087)
## [0.3.649] - 2025-06-12

### ⚙️ Miscellaneous Tasks

- Meltano airflow nix setup + use airflow postgres (#2136)
## [0.3.648] - 2025-06-12

### 🚀 Features

- Email notification (#1537)

### ⚙️ Miscellaneous Tasks

- Fixing trigger issue (#2183)
- *(dev)* Set version to 0.3.648-dev [ci skip]
- Making it explicit that cypress and bats use different login m… (#2179)
## [0.3.647] - 2025-06-11

### 🐛 Bug Fixes

- Lana manuals (#2173)
- Merge issue (#2184)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.640-dev [ci skip]
- Cypress without tilt (#2172)
- Add liquidation duration to terms (#2148)
- *(translation)* Change matured CreditFacilityStatus to Finalizado instead of Vencido (#2171)
- Some script cleanup (#2176)
- Is this unused (#2180)
- More robust hash creation (#2181)
- Adding back socket (#2182)
## [0.3.646] - 2025-06-10

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.639-dev [ci skip]
## [0.3.638] - 2025-06-10

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.636-dev [ci skip]
## [0.3.637] - 2025-06-09

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.618-dev [ci skip]
- Rename CustodianConfig -> Custodian (#2144)
- Add meltano.nix (#2147)
- Moving podman install script to makefile (#2152)
- Bump python to 3.12 for faster build (#2155)
- Fix nix dev shell caching (#2156)
- Removing legacy fn (#2154)
## [0.3.617] - 2025-06-07

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.615-dev [ci skip]
## [0.3.616] - 2025-06-06

### 🚜 Refactor

- Move terms templates (#2142)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.613-dev [ci skip]
## [0.3.613] - 2025-06-06

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.610-dev [ci skip]
## [0.3.611] - 2025-06-06

### 🐛 Bug Fixes

- Return current_cvl directly (#2143)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.607-dev [ci skip]
## [0.3.608] - 2025-06-05

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.606-dev [ci skip]
- Add 'effective' to disbursal settled event (#2114)
## [0.3.605] - 2025-06-05

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.603-dev [ci skip]
## [0.3.604] - 2025-06-04

### 🐛 Bug Fixes

- Modal reset (#2132)

### 🚜 Refactor

- Move datetime tooltip to shared web (#2126)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.598-dev [ci skip]
- Add core/custody (#2119)
## [0.3.601] - 2025-06-04

### ⚙️ Miscellaneous Tasks

- Sim-bootsrap also for the binary (#2133)
## [0.3.598] - 2025-06-04

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.595-dev [ci skip]
- Remove yanked version (#2129)
## [0.3.596] - 2025-06-03

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.591-dev [ci skip]
- Re-separate check-code rust steps (#2130)
## [0.3.593] - 2025-06-03

### ⚙️ Miscellaneous Tasks

- We no longer need rust cache as we use cachix (#2127)
- Bump flake (#2124)
## [0.3.592] - 2025-06-03

### ⚙️ Miscellaneous Tasks

- Be explicit on the flag needed (#2125)
## [0.3.591] - 2025-06-03

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.588-dev [ci skip]
- Enable all-features for musl target (#2123)
## [0.3.589] - 2025-06-03

### 🐛 Bug Fixes

- Prevent zero amount disbursals (#2120)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.587-dev [ci skip]
## [0.3.586] - 2025-06-03

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.584-dev [ci skip]
## [0.3.585] - 2025-06-03

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.582-dev [ci skip]
- Bump edition to 2024 (#2118)
## [0.3.581] - 2025-06-02

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.580-dev [ci skip]
- Removing mailhog to save on ressources (#2111)
## [0.3.580] - 2025-06-02

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.579-dev [ci skip]
- Merge check-code pipelines (#2107)
## [0.3.578] - 2025-06-02

### 🚀 Features

- Add 'effective' to public events and gql credit history entries (#2091)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.575-dev [ci skip]
## [0.3.577] - 2025-06-02

### ⚙️ Miscellaneous Tasks

- Rename origination to facility approved in facility history (#2113)
## [0.3.574] - 2025-06-02

### 🚜 Refactor

- Add COA Integrations service (#2106)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.572-dev [ci skip]
## [0.3.573] - 2025-06-01

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.570-dev [ci skip]
- Removing cache-nix (#2112)
## [0.3.569] - 2025-05-31

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.569-dev [ci skip]
- Update tower-http (#2108)
## [0.3.568] - 2025-05-30

### 🚜 Refactor

- Use Disbursal service (#2088)
- Use CreditFacilities service (#2099)
- Rename User::assign_role to User::update_role (#2094)
- Use Payments and PaymentAllocations service (#2102)
- Add effective field to Obligation entity (#2062)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.568-dev [ci skip]
- Add description and translation for permissions (#2092)
- Add PermissionSetName enum to GQL (#2093)
- Separate pages for role creation and editing (#2095)
## [0.3.567] - 2025-05-27

### 🚀 Features

- Start using Permission Set and Role entities (#2044)

### 🚜 Refactor

- Extract defaulted overdue and due record usecase (#2084)
- Extract record_collateral and create new collatreal use case (#2085)

### 🧪 Testing

- Expand repayment plan unit tests (#2065)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.567-dev [ci skip]
- Pipeline chores (#2040)
- Improve types related to roles (#2089)
## [0.3.566] - 2025-05-26

### 🚀 Features

- Roles and permissions ui (#2061)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.564-dev [ci skip]
## [0.3.565] - 2025-05-25

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.561-dev [ci skip]
- Update pipeline (#2038)
## [0.3.562] - 2025-05-24

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.558-dev [ci skip]
- Test at fixing fatal error: concurrent map writes issue (#2076)
- Remove warning (#2079)
- Remove meltano from the list of docker deps (#2078)
- Adding cargo machete (#2080)
## [0.3.559] - 2025-05-24

### ⚙️ Miscellaneous Tasks

- Updating axum and graphql (#2073)
- Updating kratos (#2072)
## [0.3.557] - 2025-05-24

### ⚙️ Miscellaneous Tasks

- Enforcing cargo deny (#2068)
- Updaging clap uuid sha2 (#2069)
## [0.3.556] - 2025-05-24

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.554-dev [ci skip]
- Updating sqlx (#2067)
## [0.3.555] - 2025-05-24

### 🚜 Refactor

- Move status transition check into entity & test (#2054)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.553-dev [ci skip]
## [0.3.552] - 2025-05-23

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.551-dev [ci skip]
- Activate facility scenarios action (#2059)
## [0.3.551] - 2025-05-23

### 🚜 Refactor

- Better interfaces for roles
## [0.3.550] - 2025-05-23

### 🚀 Features

- Role gql (#2058)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.549-dev [ci skip]
## [0.3.549] - 2025-05-23

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.546-dev [ci skip]
- Add PermissionSet to gql (#2056)
## [0.3.547] - 2025-05-23

### 🚜 Refactor

- Introduce access layer in graphql (#2053)
## [0.3.545] - 2025-05-23

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.542-dev [ci skip]
## [0.3.544] - 2025-05-22

### ⚙️ Miscellaneous Tasks

- *(seed)* Use has_outstanding_obligations in all scenarios (#2052)
## [0.3.541] - 2025-05-22

### 🚀 Features

- Seed statements module configs (#2029)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.541-dev [ci skip]
## [0.3.540] - 2025-05-22

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.538-dev [ci skip]
## [0.3.539] - 2025-05-22

### 🐛 Bug Fixes

- Incomplete bootstrapped facility (#2050)

### 🚜 Refactor

- Allow any-last-position balance-type definition (#2047)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.536-dev [ci skip]
- Make net balances signed (#2049)
- Standardize date formats and add tooltip timestamps (#2046)
- Remove premature structure from Core Access (#2051)
## [0.3.536] - 2025-05-21

### 🐛 Bug Fixes

- Script for bats test with nix (#2042)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.533-dev [ci skip]
## [0.3.534] - 2025-05-21

### 🚜 Refactor

- Rename 'Core User' into 'Core Access' (#2045)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.532-dev [ci skip]
## [0.3.531] - 2025-05-21

### 🚀 Features

- Introduce Permission Sets (#2002)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.531-dev [ci skip]
## [0.3.530] - 2025-05-20

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.529-dev [ci skip]
- Test
## [0.3.529] - 2025-05-20

### 🐛 Bug Fixes

- Nix calling name (#2041)

### ⚙️ Miscellaneous Tasks

- Simplifying dev script (#2034)
## [0.3.528] - 2025-05-20

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.526-dev [ci skip]
- Use the new image path (#2035)
## [0.3.527] - 2025-05-20

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.523-dev [ci skip]
## [0.3.524] - 2025-05-20

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.522-dev [ci skip]
- Add missing tracing for accounting module (#2039)
## [0.3.521] - 2025-05-20

### 🚀 Features

- *(seed)* Multiple scenarios (#2026)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.521-dev [ci skip]
## [0.3.520] - 2025-05-20

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.520-dev [ci skip]
- Add tracing for journal entries (#2031)
## [0.3.519] - 2025-05-20

### 📚 Documentation

- Update admin panel login URL (#2020)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.517-dev [ci skip]
- Use dev for now for nix build (#2037)
## [0.3.518] - 2025-05-20

### 📚 Documentation

- Fix browserstack env var (#2022)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.515-dev [ci skip]
## [0.3.515] - 2025-05-19

### 🧪 Testing

- Remove bats suite-level start-server (#2028)
- Extend checking account wait (#2030)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.514-dev [ci skip]
## [0.3.513] - 2025-05-19

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.512-dev [ci skip]
- Show closest account with code in journal entries (#2024)
- Remove stop_server force kill (#2027)
## [0.3.512] - 2025-05-19

### 🚀 Features

- *(seed)* Disbursal in differnt months (#2018)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.509-dev [ci skip]
## [0.3.510] - 2025-05-19

### 🚀 Features

- Seed terms in sim-bootstrap (#2019)
- Seed module configs (#2010)
## [0.3.509] - 2025-05-19

### 🐛 Bug Fixes

- Data pipeline events (#1971)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.508-dev [ci skip]
## [0.3.507] - 2025-05-17

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.507-dev [ci skip]
## [0.3.506] - 2025-05-16

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.504-dev [ci skip]
## [0.3.505] - 2025-05-16

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.502-dev [ci skip]
- Switch to payment allocation for history in customer panel (#2008)
## [0.3.502] - 2025-05-16

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.499-dev [ci skip]
- Run facility scenarios only via manual trigger and small fix (#2015)
## [0.3.500] - 2025-05-16

### ⚙️ Miscellaneous Tasks

- Add closestAccountWithCode to LedgerAccount (#2016)
## [0.3.499] - 2025-05-16

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.495-dev [ci skip]
- Remove accrualInterval and accrualCycleInterval (#2011)
- Bump flake (#2012)
## [0.3.497] - 2025-05-16

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.493-dev [ci skip]
- Show effective date in journal (#2013)
## [0.3.493] - 2025-05-16

### 🐛 Bug Fixes

- Repayment plan paid status (#2009)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.491-dev [ci skip]
## [0.3.491] - 2025-05-15

### 💼 Other

- New postgres version (#1978)
## [0.3.490] - 2025-05-15

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.490-dev [ci skip]
- Switch to payment allocation for deposit account history (#2006)
## [0.3.489] - 2025-05-15

### 🚀 Features

- Configure obligations overdue (#1999)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.488-dev [ci skip]
## [0.3.488] - 2025-05-15

### 🧪 Testing

- Charts add trial balance accounts (#2005)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.487-dev [ci skip]
## [0.3.486] - 2025-05-15

### 🚀 Features

- *(seed)* Principal late by a month (#1995)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.486-dev [ci skip]
## [0.3.485] - 2025-05-15

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.485-dev [ci skip]
- Increase complete_facility retries in retry_on_concurrent_modification (#2000)
## [0.3.484] - 2025-05-15

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.483-dev [ci skip]
- Add some missing tracing (#1996)
## [0.3.483] - 2025-05-15

### ⚙️ Miscellaneous Tasks

- Seed chart of accounts in code (#1993)
## [0.3.482] - 2025-05-15

### 🚀 Features

- Add use cases manipulating Role (#1988)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.482-dev [ci skip]
## [0.3.481] - 2025-05-14

### 🐛 Bug Fixes

- Allocation external_id has idx (#1997)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.481-dev [ci skip]
## [0.3.480] - 2025-05-14

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.480-dev [ci skip]
- Retry complete facility on concurrent modification (#1994)
## [0.3.479] - 2025-05-14

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.477-dev [ci skip]
## [0.3.478] - 2025-05-13

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.475-dev [ci skip]
- Populate repayment plan pre-activation (#1985)
## [0.3.475] - 2025-05-13

### ⚙️ Miscellaneous Tasks

- Use distroless insted of scratch (#1989)
## [0.3.474] - 2025-05-13

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.472-dev [ci skip]
## [0.3.473] - 2025-05-12

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.469-dev [ci skip]
- Nix update 2 (#1981)
## [0.3.468] - 2025-05-12

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.468-dev [ci skip]
- Add tracing for csv import (#1986)
## [0.3.467] - 2025-05-12

### 🚀 Features

- Compile with sim bootstrap (#1984)

### 🚜 Refactor

- Introduce Core User (#1982)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.463-dev [ci skip]
- Adjust facility due date (#1972)
- Nix update 2
- *(dev)* Set version to 0.3.466-dev [ci skip]
- Change flow of sim from event based to parallel time loop (#1983)
## [0.3.464] - 2025-05-11

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.462-dev [ci skip]
- Update nix
- Caching with nix
- New test
## [0.3.461] - 2025-05-10

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.461-dev [ci skip]
- Replacing health check (#1977)
## [0.3.460] - 2025-05-09

### 🧪 Testing

- Adding back some bats tests (#1975)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.460-dev [ci skip]
- Adding back health check (#1974)
- Skip again customer login for now (#1976)

### ◀️ Revert

- "ci: adding back health check (#1974)"
## [0.3.459] - 2025-05-09

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.458-dev [ci skip]
- Update effective date in relevant templates (#1957)
## [0.3.458] - 2025-05-09

### 🐛 Bug Fixes

- *(seed)* Scenario 1 (#1969)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.457-dev [ci skip]
## [0.3.456] - 2025-05-09

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.454-dev [ci skip]
- Gate release behind bats-tests (#1967)
- Removed unused rbac file (#1968)
## [0.3.455] - 2025-05-09

### ⚙️ Miscellaneous Tasks

- Record customer bats test log (#1966)
## [0.3.453] - 2025-05-09

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.453-dev [ci skip]
- Remove From<EsEntityError> for top level CoreCreditError
## [0.3.452] - 2025-05-09

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.450-dev [ci skip]
## [0.3.451] - 2025-05-09

### 🐛 Bug Fixes

- *(reporting)* Filter stale rows from raw tables (#1958)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.447-dev [ci skip]
## [0.3.448] - 2025-05-08

### 🚜 Refactor

- Remove redundant rbac (#1962)
## [0.3.447] - 2025-05-08

### 🐛 Bug Fixes

- Date-range-picker to use date instead of timestamps (#1960)
- Test lints

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.445-dev [ci skip]
## [0.3.445] - 2025-05-08

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.444-dev [ci skip]
- In RBAC hierarchy call 'parent' the more powerful role (#1961)
## [0.3.443] - 2025-05-08

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.441-dev [ci skip]
## [0.3.442] - 2025-05-08

### 🚜 Refactor

- Allow deposit account to be created on customer activation (#1951)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.440-dev [ci skip]
## [0.3.439] - 2025-05-08

### 🚜 Refactor

- Rename balance range fields to align with Cala (#1959)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.437-dev [ci skip]
## [0.3.438] - 2025-05-07

### 🐛 Bug Fixes

- Trial balance zero balance check (#1948)

### 🧪 Testing

- Fix accounting.bats

### ⚙️ Miscellaneous Tasks

- Sqlx-prepare
## [0.3.436] - 2025-05-07

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.435-dev [ci skip]
- Switch to using effective balances in ranges (#1954)
## [0.3.435] - 2025-05-07

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.431-dev [ci skip]
## [0.3.433] - 2025-05-06

### 🐛 Bug Fixes

- Entry type mapping for disbursal in transactions (#1946)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.424-dev [ci skip]
- Dont use sim bootstrap in built image (#1949)
- Try update on oathkeeper proxy (#1944)
## [0.3.423] - 2025-05-03

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.422-dev [ci skip]
- Unused command (#1945)
## [0.3.422] - 2025-05-03

### ⚙️ Miscellaneous Tasks

- Remove link in docker-compose (#1943)
## [0.3.421] - 2025-05-03

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.421-dev [ci skip]
- .sqlx in sync (#1942)
## [0.3.420] - 2025-05-02

### 🚀 Features

- Facility scenarios using sim-bootstrap (#1890)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.420-dev [ci skip]
## [0.3.419] - 2025-05-02

### 🚜 Refactor

- Include InterestPeriod in AccrualCycles (#1933)
- Remove existing_obligations vec & related types (#1939)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.419-dev [ci skip]
## [0.3.418] - 2025-05-02

### 🚀 Features

- *(reporting)* Dbt macro to drop stale tables (#1925)

### ⚙️ Miscellaneous Tasks

- Rename credit facility transaction to history (#1937)
- Add missing transactions to P&L and balance sheet (#1938)
- *(dev)* Set version to 0.3.418-dev [ci skip]
## [0.3.417] - 2025-05-02

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.415-dev [ci skip]
- Repayment plan persistent projection (#1932)
## [0.3.416] - 2025-05-02

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.410-dev [ci skip]
- Expose audit entry id field (#1935)
## [0.3.413] - 2025-05-02

### ⚙️ Miscellaneous Tasks

- Remove redundant commands for from cypress (#1934)
## [0.3.411] - 2025-05-02

### 🐛 Bug Fixes

- Collateralization_ratio divide by 0

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.407-dev [ci skip]
- Add missing translation for credit-facilities (#1929)
## [0.3.408] - 2025-05-01

### 🚀 Features

- Add disbursed defaulted job (#1898)

### 🚜 Refactor

- Add Credit Facility history job (#1927)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.404-dev [ci skip]
## [0.3.405] - 2025-05-01

### 🚜 Refactor

- Rename customer-onboarding to customer-sync (#1926)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.403-dev [ci skip]
## [0.3.402] - 2025-05-01

### 🚀 Features

- Introduce Collateral entity (#1849)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.399-dev [ci skip]
## [0.3.401] - 2025-05-01

### 🚀 Features

- Reg report nrp 51 (#1803)
## [0.3.398] - 2025-05-01

### 🚀 Features

- Reg report xml validation script (#1893)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.398-dev [ci skip]
## [0.3.397] - 2025-04-30

### 🐛 Bug Fixes

- Persona report (#1895)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.397-dev [ci skip]
## [0.3.396] - 2025-04-30

### 🚀 Features

- Update client email (#1920)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.395-dev [ci skip]
## [0.3.395] - 2025-04-30

### 📚 Documentation

- Update README.md (#1922)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.393-dev [ci skip]
- Improve the pipeline (#1921)
## [0.3.393] - 2025-04-29

### 🚜 Refactor

- Consistent payment allocation (#1917)
## [0.3.392] - 2025-04-29

### 🚜 Refactor

- Add CreateCreditFacility template (#1913)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.391-dev [ci skip]
## [0.3.391] - 2025-04-29

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.390-dev [ci skip]
- Add public events for Obligation (#1916)
## [0.3.389] - 2025-04-29

### 🐛 Bug Fixes

- Fallback to account ID in links if account code is missing (#1915)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.386-dev [ci skip]
## [0.3.388] - 2025-04-29

### 🚜 Refactor

- Add balance detail type to expose debit and credit (#1910)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.385-dev [ci skip]
## [0.3.384] - 2025-04-29

### 📚 Documentation

- Misc Update README.md (#1914)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.382-dev [ci skip]
## [0.3.383] - 2025-04-28

### 🐛 Bug Fixes

- Check-code sdl (#1911)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.380-dev [ci skip]
## [0.3.380] - 2025-04-28

### 🐛 Bug Fixes

- Ignore due/overdue record if paid (#1909)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.379-dev [ci skip]
## [0.3.378] - 2025-04-28

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.377-dev [ci skip]
## [0.3.377] - 2025-04-28

### 🐛 Bug Fixes

- Ci regressions (#1907)
## [0.3.376] - 2025-04-28

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.375-dev [ci skip]
- Early return from due/overdue jobs if paid (#1899)
## [0.3.375] - 2025-04-28

### 🚜 Refactor

- Pub(super) is no longer needed on events (#1905)
- Move CVL to CreditFacilitySummaryBalances (#1903)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.367-dev [ci skip]
- New test
- Test
- Testing scratch source image
## [0.3.366] - 2025-04-26

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.361-dev [ci skip]
- Some gh updates (#1900)
## [0.3.362] - 2025-04-26

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.358-dev [ci skip]
## [0.3.359] - 2025-04-25

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.356-dev [ci skip]

### ◀️ Revert

- "wip"
## [0.3.355] - 2025-04-25

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.355-dev [ci skip]
## [0.3.354] - 2025-04-25

### 🐛 Bug Fixes

- Facility balance totals (#1881)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.354-dev [ci skip]
- Publish CoreDepositEvent DepositAccountCreated
- Remove interest overdue state (#1885)
- Use only disbursed amount when calculating interest (#1884)
- Update next to v15 (#1894)
## [0.3.353] - 2025-04-25

### 🚜 Refactor

- Balance range by currency struct in gql  (#1855)

### ⚙️ Miscellaneous Tasks

- Misc toml makefile update (#1891)
- *(dev)* Set version to 0.3.352-dev [ci skip]

### ◀️ Revert

- Allocate heap memory to term values in cf (#1858)
## [0.3.352] - 2025-04-25

### 🐛 Bug Fixes

- Add dbt-dev.tf

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.350-dev [ci skip]
## [0.3.349] - 2025-04-24

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.345-dev [ci skip]
- Remove bc dependency in flake (#1877)
- Update jsonwebtoken (#1883)
- Upgrading rand (#1879)
## [0.3.348] - 2025-04-24

### 🐛 Bug Fixes

- Move payment recorded event (#1832)
## [0.3.346] - 2025-04-24

### ⚙️ Miscellaneous Tasks

- Also grouping dev dependencies PR (#1871)
## [0.3.345] - 2025-04-24

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.341-dev [ci skip]
## [0.3.343] - 2025-04-24

### ⚙️ Miscellaneous Tasks

- Ignore nextjs and react updates (#1872)
## [0.3.340] - 2025-04-24

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.337-dev [ci skip]
- Attempt to fix storage (#1864)
- Adding dependabot for pnpm (#1861)
## [0.3.336] - 2025-04-23

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.334-dev [ci skip]
- Simplifying script (#1860)
## [0.3.335] - 2025-04-23

### ⚙️ Miscellaneous Tasks

- Skip docs-upload and netlify deploy (#1857)
## [0.3.334] - 2025-04-23

### 🚀 Features

- Show all accounts with accountcode and balances in trial balance (#1854)
- *(ui)* Show ancestors and children in ledger account details (#1856)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.332-dev [ci skip]

### ◀️ Revert

- Is it still necessary to rm stuff (#1853) (#1859)
## [0.3.332] - 2025-04-23

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.330-dev [ci skip]
- Is it still necessary to rm stuff (#1853)
- Cache optimization (#1852)
## [0.3.330] - 2025-04-23

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.329-dev [ci skip]
- Misc concourse cleanup (#1851)
## [0.3.328] - 2025-04-23

### 🐛 Bug Fixes

- *(admin-panel)* Flicker on export (#1848)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.327-dev [ci skip]
- Error if deposit/withdrawal amount is zero (#1850)
## [0.3.327] - 2025-04-23

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.324-dev [ci skip]
- Simplifying github actions (#1845)
## [0.3.325] - 2025-04-22

### 🧪 Testing

- Facility record payment e2e (#1820)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.322-dev [ci skip]
- Remove auto-focus from tables (#1847)
## [0.3.322] - 2025-04-22

### 🧪 Testing

- Make tests pass if GCP env var is not present (#1840)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.321-dev [ci skip]
## [0.3.320] - 2025-04-21

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.320-dev [ci skip]
- *(reporting)* Minor improvements to produce non-regulatory reports in holistics (#1830)
## [0.3.319] - 2025-04-21

### 🐛 Bug Fixes

- Replaced Logo with Updated Design (#1755)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.317-dev [ci skip]
- Remove cash flow and statement structs (#1838)
## [0.3.318] - 2025-04-18

### 🐛 Bug Fixes

- Disbursed amount updating in dashboard (#1821)
## [0.3.316] - 2025-04-18

### 🚀 Features

- Export csv ui (#1828)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.316-dev [ci skip]
## [0.3.315] - 2025-04-18

### ⚡ Performance

- Clean up space of integration test (#1839)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.315-dev [ci skip]
## [0.3.314] - 2025-04-18

### 🚜 Refactor

- Simplify init terraform

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.314-dev [ci skip]
## [0.3.313] - 2025-04-18

### 🐛 Bug Fixes

- Trial balance integration test (#1833)

### 🚜 Refactor

- Move trial balance to core/accounting (#1822)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.313-dev [ci skip]
## [0.3.312] - 2025-04-17

### 🐛 Bug Fixes

- Broken facility history from failing 'expect' (#1816)

### 🧪 Testing

- Remove flaky assert in gcs.rs (#1827)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.312-dev [ci skip]
## [0.3.311] - 2025-04-17

### 🚜 Refactor

- Payment allocation entity (#1824)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.309-dev [ci skip]
- Bump flake
## [0.3.310] - 2025-04-17

### 🚀 Features

- *(ui)* Tx templates (#1807)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.307-dev [ci skip]
## [0.3.307] - 2025-04-17

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.305-dev [ci skip]
- Caching and upload videos for failed (#1825)
## [0.3.305] - 2025-04-16

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.304-dev [ci skip]

### ◀️ Revert

- "chore(deps): bump sqlx from 0.8.3 to 0.8.5 (#1819)"
## [0.3.303] - 2025-04-16

### 🧪 Testing

- Add more collateralization_ratio coverage (#1817)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.303-dev [ci skip]
## [0.3.302] - 2025-04-16

### 🚜 Refactor

- Cvl check fn signatures to bool (#1814)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.301-dev [ci skip]
- *(cypress)* Use larger runner for local headless test (#1808)
## [0.3.301] - 2025-04-16

### 🚀 Features

- Export csv (#1780)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.300-dev [ci skip]
## [0.3.299] - 2025-04-16

### 🚜 Refactor

- Construct LedgerAccount in gql layer (#1810)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.299-dev [ci skip]
## [0.3.298] - 2025-04-16

### 🐛 Bug Fixes

- Return account set if code parses (#1813)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.298-dev [ci skip]
## [0.3.297] - 2025-04-16

### 🚀 Features

- Reg report nrsf-03 04 titulares (#1791)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.297-dev [ci skip]
- *(reporting)* Minor changes for persona.xml (#1815)
## [0.3.296] - 2025-04-15

### 🐛 Bug Fixes

- Facility completion balance checks (#1812)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.296-dev [ci skip]
## [0.3.295] - 2025-04-15

### 🚀 Features

- Add Obligations "record payment" and switch use-case (#1752)

### 🐛 Bug Fixes

- Action value for pnl coa config (#1811)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.294-dev [ci skip]
## [0.3.294] - 2025-04-14

### 🚀 Features

- Introduce ledgerTransactionsForTemplateCode GQL (#1794)

### 🚜 Refactor

- Move balance sheet to core/accounting (#1801)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.291-dev [ci skip]
## [0.3.292] - 2025-04-14

### 🚀 Features

- Reg report nrsf 03 05 agencies (#1792)
- Add GraphQL query transactionTemplates (#1800)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.288-dev [ci skip]
- Allow to set effective date when creating manual transaction (#1799)
- Restore statement init for pnl, balance sheet creation (#1802)
## [0.3.289] - 2025-04-12

### 🚀 Features

- Reg report nrsf 03 07 bank employees (#1793)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.287-dev [ci skip]
## [0.3.286] - 2025-04-11

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.285-dev [ci skip]
- Remove unused statements (#1790)
## [0.3.285] - 2025-04-11

### 🚀 Features

- *(ui)* Execute manual transaction (#1786)

### 🚜 Refactor

- Move pnl to core/accounting (#1764)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.282-dev [ci skip]
## [0.3.283] - 2025-04-10

### 🚀 Features

- Deposit Accounts Report (NSRF-03)  (#1785)

### 🐛 Bug Fixes

- *(reporting)* Reduce the frequency for some meltano jobs (#1789)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.281-dev [ci skip]
## [0.3.280] - 2025-04-10

### 🐛 Bug Fixes

- Algo for find_leaf_children fn (#1787)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.280-dev [ci skip]
## [0.3.279] - 2025-04-10

### 🚀 Features

- *(ui)* Ledger transactions (#1783)
- Ledger account children (#1784)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.277-dev [ci skip]
## [0.3.278] - 2025-04-10

### 🐛 Bug Fixes

- *(ui)* Disable next page in paginated tables when loading (#1781)

### 🧪 Testing

- Add end-to-end test for manual transactions (#1782)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.274-dev [ci skip]
- Bump deps
## [0.3.275] - 2025-04-09

### 🚀 Features

- *(reporting)* Download document images from Sumsub API (#1767)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.272-dev [ci skip]
## [0.3.272] - 2025-04-09

### 🚀 Features

- Make Manual Transactions accessible via GraphQL (#1769)
## [0.3.271] - 2025-04-09

### 🐛 Bug Fixes

- *(ui)* Journal pagination (#1778)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.265-dev [ci skip]
## [0.3.270] - 2025-04-09

### ⚙️ Miscellaneous Tasks

- Fail if there are high or critical issues (#1774)
## [0.3.265] - 2025-04-09

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.264-dev [ci skip]
- LedgerTransaction (#1776)
## [0.3.263] - 2025-04-09

### 🚜 Refactor

- Move cloud-storage to lib (#1771)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.263-dev [ci skip]
## [0.3.262] - 2025-04-08

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.262-dev [ci skip]
- Removing debug info (#1773)
## [0.3.261] - 2025-04-08

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.261-dev [ci skip]
- Adding CodeQL to rust (#1772)
## [0.3.260] - 2025-04-08

### 🚀 Features

- Add support for Manual Transactions (#1759)

### 🐛 Bug Fixes

- Minimized scrollbar in Menu (#1761)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.260-dev [ci skip]
## [0.3.259] - 2025-04-08

### 🐛 Bug Fixes

- Capitalize meta title in customer portal (#1754)

### 🚜 Refactor

- Add balance range to ledger account (#1765)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.257-dev [ci skip]
## [0.3.258] - 2025-04-06

### 🐛 Bug Fixes

- Do not authorize disbursment if customer is not active (#1762)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.253-dev [ci skip]
- Add account name in journal (#1760)
- Show ancestors / navigate in LedgerAccount details view (#1758)
- Remove dashboard from breadcrumb  (#1757)
## [0.3.255] - 2025-04-04

### 🚜 Refactor

- Consistent use of CalaAccountId type

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.251-dev [ci skip]
## [0.3.251] - 2025-04-04

### ⚙️ Miscellaneous Tasks

- Make Ledger Account history return both leaf accounts and sets (#1751)
## [0.3.250] - 2025-04-04

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.249-dev [ci skip]
- LedgerAccount ancestors (#1745)
## [0.3.249] - 2025-04-03

### 🚀 Features

- Add Obligation due events (#1747)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.248-dev [ci skip]
## [0.3.247] - 2025-04-03

### 🚀 Features

- Introduce new `Obligation` entity (#1726)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.247-dev [ci skip]
## [0.3.246] - 2025-04-03

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.244-dev [ci skip]
- Use --turbo for nextjs to lunch faster (#1728)
- Sort trial balance by account code (#1743)
## [0.3.245] - 2025-04-03

### 🚜 Refactor

- Use Journal Entry in Account History result (#1721)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.242-dev [ci skip]
## [0.3.242] - 2025-04-02

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.240-dev [ci skip]
- *(ui)* Connect journal with ledger (#1736)
- Fix uploads to netlify (#1744)
## [0.3.240] - 2025-04-02

### 🐛 Bug Fixes

- Missing repayment in repayment plan (#1741)

### ⚙️ Miscellaneous Tasks

- Add journal-related permissions to authorization seed (#1740)
- *(dev)* Set version to 0.3.239-dev [ci skip]
- Fix textual representation of JournalError (#1742)
## [0.3.238] - 2025-04-02

### 🚀 Features

- Reg report nrsf 03 (#1720)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.238-dev [ci skip]
## [0.3.237] - 2025-04-02

### ⚙️ Miscellaneous Tasks

- Autofill btn for config modules (#1709)
- *(dev)* Set version to 0.3.236-dev [ci skip]
- Bump Cala version (#1734)
## [0.3.236] - 2025-04-01

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.234-dev [ci skip]
- Ui general-ledger -> journal (#1723)
- Interest based on 365 days (#1704)
## [0.3.234] - 2025-04-01

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.233-dev [ci skip]
- Restore frontend check-code in acitons (#1729)
## [0.3.232] - 2025-04-01

### 🐛 Bug Fixes

- Credit Facility test (#1732)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.230-dev [ci skip]
- Ledger_account by id query (#1719)
## [0.3.231] - 2025-03-31

### 🚜 Refactor

- Rename accrual/incurrence to accrualCycle/accrual (#1725)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.226-dev [ci skip]
## [0.3.228] - 2025-03-30

### ⚙️ Miscellaneous Tasks

- Gh codeql action (#677)
## [0.3.225] - 2025-03-30

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.225-dev [ci skip]
- *(shared)* Bump vendored ci files (#1235)
## [0.3.224] - 2025-03-30

### 🚜 Refactor

- Move js lib to apps (#1724)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.220-dev [ci skip]
- Documentation on bug that needs fixing (#1687)
## [0.3.223] - 2025-03-28

### 🐛 Bug Fixes

- *(reporting)* Sql mistake causing tap-sumsubapi to fail (#1722)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.218-dev [ci skip]
## [0.3.217] - 2025-03-28

### 🚜 Refactor

- Move general ledger into core/accounting as journal (#1710)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.214-dev [ci skip]
- Update flake and add mermaid-cli (#1715)
## [0.3.216] - 2025-03-27

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.213-dev [ci skip]
- Upgrade otel agent (#1714)
- Upgrade kratos (#1712)
## [0.3.212] - 2025-03-27

### 🚜 Refactor

- Introduce core accounting (#1708)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.212-dev [ci skip]
## [0.3.211] - 2025-03-26

### 🚀 Features

- Add disbursed overdue job for credit facilities (#1693)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.210-dev [ci skip]
## [0.3.210] - 2025-03-25

### 🚀 Features

- Add opening and closing balance ui (#1694)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.208-dev [ci skip]
- Get rid of unused tables (#1703)
- Startup without gcp creds (#1698)
## [0.3.208] - 2025-03-25

### 🚀 Features

- Interest receivables account sets (#1697)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.205-dev [ci skip]
- Add missing module config sample codes (#1696)
## [0.3.206] - 2025-03-25

### 🚀 Features

- Expose general ledger to admin panel (#1689)

### 🚜 Refactor

- Notify outbox fn (#1699)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.203-dev [ci skip]
## [0.3.203] - 2025-03-24

### 🚀 Features

- Pnl module config ui (#1683)
- Short and long term disbursed receivable accounts (#1686)

### 🐛 Bug Fixes

- Do not double add sets to trial balance (#1691)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.200-dev [ci skip]
## [0.3.201] - 2025-03-21

### 🐛 Bug Fixes

- Ledger history amount should be in dollars (#1684)

### 🚜 Refactor

- Add internal account sets for deposit (#1685)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.197-dev [ci skip]
## [0.3.198] - 2025-03-21

### 🐛 Bug Fixes

- Add paging for find_report_outputs (#1672)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.196-dev [ci skip]
## [0.3.195] - 2025-03-21

### 🚀 Features

- Add profit and loss configure (#1677)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.194-dev [ci skip]
## [0.3.194] - 2025-03-21

### 🚜 Refactor

- Make outstanding amounts more granular (#1510)

### 🧪 Testing

- Modules manuals (#1669)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.191-dev [ci skip]
## [0.3.192] - 2025-03-20

### 🚜 Refactor

- Paginated trial balance gql query (#1650)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.188-dev [ci skip]
## [0.3.189] - 2025-03-20

### 🚜 Refactor

- Use process managers to sync txs to sumsub (#1633)

### ⚙️ Miscellaneous Tasks

- Balance sheet config ui (#1668)
## [0.3.187] - 2025-03-20

### 🚀 Features

- Add balance sheet configure (#1661)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.186-dev [ci skip]
## [0.3.186] - 2025-03-20

### 🚜 Refactor

- Clean-up customer create dialog (#1660)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.181-dev [ci skip]
- Add sample codes for module configs (#1634)
- Admin-panel ui items (#1642)
- Remove dots in module config input (#1666)
## [0.3.184] - 2025-03-19

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.180-dev [ci skip]
- Not waiting for core to compile pnpm apps (#1655)
- Remove login to gcr step in gh action workflows (#1659)
- Disable create button if customer inactive (#1640)
## [0.3.179] - 2025-03-18

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.177-dev [ci skip]
- *(customer-portal)* Update information card text (#1641)
- Probe for consumer app (#1654)
## [0.3.178] - 2025-03-18

### ⚙️ Miscellaneous Tasks

- Change account_code type from String to AccountCode (#1639)
## [0.3.176] - 2025-03-18

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.174-dev [ci skip]
## [0.3.175] - 2025-03-18

### 🚀 Features

- Add code field for trial balance accounts (#1636)
## [0.3.174] - 2025-03-18

### 🐛 Bug Fixes

- Minor data pipeline issues (#1627)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.172-dev [ci skip]
- Add missing steps in customer manuals (#1626)
## [0.3.172] - 2025-03-17

### 🚀 Features

- Expand credit internal account sets (#1622)

### 🐛 Bug Fixes

- Add missing status update permission to superuser
- Chart of accounts test again

### 🧪 Testing

- Fix chart_of_accounts test

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.168-dev [ci skip]
- Spanish manuals (#1625)
- Remove redundant line from manuals
## [0.3.170] - 2025-03-17

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.167-dev [ci skip]
## [0.3.166] - 2025-03-15

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.166-dev [ci skip]
- Debit credit parsing logic in chart of accounts (#1623)
## [0.3.165] - 2025-03-15

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.165-dev [ci skip]
- Prohibit deposits when customer not active (#1620)
## [0.3.164] - 2025-03-14

### 🚜 Refactor

- Trial balance (#1619)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.163-dev [ci skip]
## [0.3.163] - 2025-03-14

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.161-dev [ci skip]
- Adding fine grained company type (#1618)
## [0.3.161] - 2025-03-14

### 🐛 Bug Fixes

- Manuals screenshot when loading (#1610)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.158-dev [ci skip]
- Ledger account balance (#1611)
## [0.3.159] - 2025-03-14

### 🚀 Features

- Ledger account details page (#1606)
- Implement ledger account details query (#1607)

### 🚜 Refactor

- Rename AccountAmounts to AccountBalanceAmounts in gql layer (#1609)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.155-dev [ci skip]
- Remove unused file (#1608)
## [0.3.156] - 2025-03-13

### 🚀 Features

- Add amount field to LedgerAccountHistoryEntry gql type (#1605)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.152-dev [ci skip]
## [0.3.153] - 2025-03-13

### ⚙️ Miscellaneous Tasks

- Boilerplate for ledger_account (#1603)
## [0.3.151] - 2025-03-13

### 🚀 Features

- Integration of deposit with sumsub (#1602)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.147-dev [ci skip]
## [0.3.150] - 2025-03-13

### 🚀 Features

- [**breaking**] Genericize chart of accounts structure (#1574)
## [0.3.147] - 2025-03-13

### 🚀 Features

- Posting tx detail to sumsub (#1521)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.146-dev [ci skip]
## [0.3.145] - 2025-03-12

### 🐛 Bug Fixes

- Makefile issues (#1596)

### 💼 Other

- Update cursor definition (#1562)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.145-dev [ci skip]
## [0.3.144] - 2025-03-12

### 🚀 Features

- *(reporting)* Update kyc details when edited on sumsub (#1558)
- Initialise new chart in seed (#1575)

### 🐛 Bug Fixes

- *(audit)* Move outdated cloud-storage -> gcp package (#1564)
- Handle parsing when parent has multiple child nodes (#1586)

### 📚 Documentation

- Add dependencies (#1577)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.141-dev [ci skip]
- *(reporting)* Sync sumsub data every two minutes (#1560)
- Make it running without GCP env (#1471)
- Remove chartOfAccountsCreate mutation (#1581)
- *(reporting)* Dummy tables for irrelevant reports (#1576)
- Translation gh action (#1515)
- Fix concourse e2e test (#1593)
## [0.3.143] - 2025-03-06

### 🚀 Features

- Add corporation client type (#1543)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.138-dev [ci skip]
- Deposit install boilerplate (#1555)
- Update readme (#1551)
## [0.3.139] - 2025-03-06

### 🚀 Features

- Add new charts gql query (#1550)
## [0.3.138] - 2025-03-06

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.137-dev [ci skip]
- Update sqlx (#1554)
## [0.3.136] - 2025-03-06

### 🚜 Refactor

- Genericize new charts structure (#1549)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.136-dev [ci skip]
## [0.3.135] - 2025-03-05

### 🚜 Refactor

- Separate jobs customer onboarding (#1542)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.135-dev [ci skip]
## [0.3.134] - 2025-03-05

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.134-dev [ci skip]
- New chart of accounts (#1538)
## [0.3.133] - 2025-03-04

### 🐛 Bug Fixes

- Instrument name (#1539)
- *(reporting)* Average open price should not be zero (#1513)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.132-dev [ci skip]
- *(cypress)* More reliable type committee name (#1531)
- *(reporting)* Fill-in missing pii fields for persona.xml (#1534)
## [0.3.132] - 2025-03-03

### 🐛 Bug Fixes

- Check code (#1532)

### 🧪 Testing

- Cypress e2e errors (#1516)
- *(e2e)* Cypress edge case for withdrawal cancel transaction (#1528)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.125-dev [ci skip]
- Validate-translations check (#1514)
- Translation for breadcrumbs (#1522)
- Update flake (#1526)
## [0.3.130] - 2025-02-26

### 🚀 Features

- Add non-cash adjustments to statement (#1426)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.121-dev [ci skip]
- Auto detect user language (#1511)
## [0.3.123] - 2025-02-26

### 🚀 Features

- Translation for admin panel (#1501)
## [0.3.121] - 2025-02-26

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.120-dev [ci skip]
- Rm credit facility (#1509)
## [0.3.119] - 2025-02-25

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.118-dev [ci skip]
- Optional deposit account (#1500)
## [0.3.118] - 2025-02-25

### 🚀 Features

- Integrate core/credit (#1489)

### 🚜 Refactor

- Change 'expire' to 'mature' in facility (#1494)

### ⚙️ Miscellaneous Tasks

- Update favicon (#1498)
- *(dev)* Set version to 0.3.114-dev [ci skip]
## [0.3.116] - 2025-02-24

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.111-dev [ci skip]
- Run cypress tests always (#1499)
## [0.3.112] - 2025-02-24

### 🚀 Features

- Admin panel loader (#1490)

### 🚜 Refactor

- Flip dependency deposit -> customer (#1483)

### 🧪 Testing

- Use random omnibus account references (#1491)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.109-dev [ci skip]
## [0.3.109] - 2025-02-22

### 🐛 Bug Fixes

- Duplicate module-ledger accounts (#1427)

### 🚜 Refactor

- Bring back core_customer (#1479)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.108-dev [ci skip]
## [0.3.107] - 2025-02-21

### 🚜 Refactor

- Remove accounting_init dependency from credit_ledger (#1482)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.107-dev [ci skip]
## [0.3.106] - 2025-02-21

### 🚜 Refactor

- Move credit-facility to core (#1453)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.106-dev [ci skip]
## [0.3.105] - 2025-02-20

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.104-dev [ci skip]
- Remove hasSubAccounts from schema (#1402)
## [0.3.104] - 2025-02-20

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.103-dev [ci skip]
- Remove redunant errored.tfstate
## [0.3.102] - 2025-02-20

### 🚀 Features

- *(reporting)* Collect data test results in a single table (#1468)

### 🐛 Bug Fixes

- *(reporting)* Revery authorized view config (#1452)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.102-dev [ci skip]
- *(reporting)* Enable INFORMATION_SCHEMA.JOBS view (#1472)
- *(reporting)* Sqlfluff (#1469)
- Move chart of accounts to administration
- Remove statements returns Option (#1403)
- Update pnpm lock (#1475)
- Set project on google_bigquery_dataset_access resource
## [0.3.101] - 2025-02-19

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.101-dev [ci skip]
- *(reporting)* Remove remaining dataform files and configuration (#1447)
## [0.3.100] - 2025-02-19

### 🚀 Features

- *(reporting)* Data tests for alternative credit facility tables (#1465)

### 🚜 Refactor

- Make charts gql type into an entity (#1456)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.100-dev [ci skip]
- *(reporting)* Configure dbt data tests to run daily and store results (#1449)
- *(reporting)* Turn off full-refresh for cached tables (#1464)
## [0.3.99] - 2025-02-19

### 🚜 Refactor

- Move price to core (#1466)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.99-dev [ci skip]
- Enable tracing across outbox handling (#1467)
## [0.3.98] - 2025-02-19

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.98-dev [ci skip]
- Add CoreCustomerEvent::CustomerAccountStatusUpdate (#1455)
## [0.3.97] - 2025-02-18

### 🚀 Features

- Cash-flow ui (#1454)

### 📚 Documentation

- Clarify that TF_VAR_sa_creds is mandatory currently (#1461)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.96-dev [ci skip]
## [0.3.96] - 2025-02-18

### 🧪 Testing

- *(bats)* Customer (#1362)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.95-dev [ci skip]
## [0.3.94] - 2025-02-18

### 🐛 Bug Fixes

- *(reporting)* Correct name for tf variable (#1450)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.93-dev [ci skip]
## [0.3.93] - 2025-02-18

### 🚀 Features

- *(reporting)* Enable live views for holistics (#1445)

### ⚙️ Miscellaneous Tasks

- *(reporting)* Ignore meltano/output directory (#1448)
- *(reporting)* Cache market data from bitfinex (#1444)
## [0.3.92] - 2025-02-18

### 🚀 Features

- Kpi short name clarification (#1382)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.92-dev [ci skip]
## [0.3.91] - 2025-02-17

### 🐛 Bug Fixes

- Handel null date in credit facility (#1439)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.89-dev [ci skip]
- Remove ApprovalProcessStarted event from credit facility (#1440)
## [0.3.90] - 2025-02-17

### 🚜 Refactor

- Avoid referencing checking account in facility templates (#1437)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.88-dev [ci skip]
## [0.3.87] - 2025-02-17

### 🚜 Refactor

- Remove legacy tf commands (#1436)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.87-dev [ci skip]
## [0.3.86] - 2025-02-17

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.86-dev [ci skip]
- *(reporting)* Enable incremental refresh for staging tables (#1431)
## [0.3.85] - 2025-02-14

### 🐛 Bug Fixes

- *(reporting)* Add rows for days with no events in daily_credit_facility_states (#1414)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.85-dev [ci skip]
## [0.3.84] - 2025-02-14

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.82-dev [ci skip]
- Bump flake (#1422)
## [0.3.83] - 2025-02-14

### 🐛 Bug Fixes

- Cvl status text in collateral card (#1421)

### ⚙️ Miscellaneous Tasks

- Transaction history in UI (#1420)
- Current cvl status (#1417)
- *(dev)* Set version to 0.3.80-dev [ci skip]
- Add back cash flow statement (#1412)
## [0.3.80] - 2025-02-14

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.76-dev [ci skip]
- Update fields (#1419)
## [0.3.78] - 2025-02-14

### 🐛 Bug Fixes

- Do not use for_subject in credit_facility

### ⚙️ Miscellaneous Tasks

- Payment query for deposit history (#1415)
- Trigger rollout on ui changes
- Add deposit account history for admin gql (#1416)
- *(dev)* Set version to 0.3.74-dev [ci skip]
- Remove for subject for payment (#1418)
## [0.3.74] - 2025-02-14

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.72-dev [ci skip]
- Update facility card labels (#1413)
## [0.3.72] - 2025-02-13

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.71-dev [ci skip]
- *(reporting)* Configure incremental updates for staging models (#1404)
## [0.3.70] - 2025-02-13

### 🚀 Features

- Payment entity (#1409)

### 🐛 Bug Fixes

- Data-table auto-focus (#1410)

### 🚜 Refactor

- Enable ranges for statement queries (#1401)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.69-dev [ci skip]
- Cf page add collateral and facility card (#1405)
- Remove kyc and balance details from tabs groups (#1408)
- Bump cala (#1400)
- Change terms modal heading
## [0.3.69] - 2025-02-12

### 🚜 Refactor

- Fetch statement balances in single query (#1380)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.68-dev [ci skip]
## [0.3.67] - 2025-02-12

### 🐛 Bug Fixes

- Filter to exclude Ignored entries (#1399)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.67-dev [ci skip]
## [0.3.66] - 2025-02-12

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.66-dev [ci skip]
- Expose disbursal for disbursal entry (#1398)
## [0.3.65] - 2025-02-12

### 🚀 Features

- DepositAccountHistory (#1390)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.65-dev [ci skip]
## [0.3.64] - 2025-02-12

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.64-dev [ci skip]
- *(reporting)* Misc updates for alternative analytical tables (#1397)
## [0.3.63] - 2025-02-11

### 🐛 Bug Fixes

- Prevent tab navigation when modifier keys are pressed
- *(reporting)* Materialize alternative holistics tables (#1394)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.63-dev [ci skip]
## [0.3.62] - 2025-02-11

### 🚀 Features

- Repayment plan for admin-panel (#1386)
- *(reporting)* Alternative analytical table design (#1389)

### ⚙️ Miscellaneous Tasks

- Remove kyc message (#1387)
- *(dev)* Set version to 0.3.62-dev [ci skip]
## [0.3.61] - 2025-02-07

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.61-dev [ci skip]
- *(apps)* Remove logs
- Add ledger_transaction_id to Deposit entity
- Disbursal query for customer (#1381)
- Remove pending balance (#1384)
- Bump cala
## [0.3.60] - 2025-02-07

### 🚀 Features

- *(reporting)* Trades and order book streams for bitfinex api extractor (#1379)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.60-dev [ci skip]
- *(reporting)* Garantia_pignorada.xml dbt migration (#1378)
## [0.3.59] - 2025-02-06

### 🐛 Bug Fixes

- Permission for list deposit account (#1377)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.58-dev [ci skip]
- *(customer-portal)* Logs for debugging
## [0.3.58] - 2025-02-06

### 🚀 Features

- Customer portal ui (#1373)

### 🐛 Bug Fixes

- *(apps)* Kratos path customer portal ssr (#1376)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.56-dev [ci skip]
## [0.3.56] - 2025-02-06

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.54-dev [ci skip]

### ◀️ Revert

- *(apps)* Customer portal basepath just like admin panel
## [0.3.54] - 2025-02-06

### 🐛 Bug Fixes

- *(apps)* Customer portal basepath just like admin panel (#1371)

### 🚜 Refactor

- Simplify statement  'find_or_create' functions to 'create' (#1370)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.52-dev [ci skip]
- Pipeline cust portal (#1372)
- Balance and status field for credit facility (#1367)
- Facility by id query for customer (#1374)
## [0.3.52] - 2025-02-05

### 🚀 Features

- Dbt collateral models (#1360)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.51-dev [ci skip]
## [0.3.50] - 2025-02-05

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.50-dev [ci skip]
- Bump cala (#1366)
## [0.3.49] - 2025-02-05

### 🚀 Features

- Auth for customer portal (#1363)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.49-dev [ci skip]
## [0.3.48] - 2025-02-05

### 🚀 Features

- Credit facilities query for customer  (#1364)

### 🐛 Bug Fixes

- Make statements find-or-create operations atomic (#1347)
- *(auth)* Into customer

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.47-dev [ci skip]
## [0.3.47] - 2025-02-05

### ⚙️ Miscellaneous Tasks

- Add DepositsForSubject (#1351)
## [0.3.46] - 2025-02-05

### 🚀 Features

- Kratos for customers (#1355)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.46-dev [ci skip]
- Customer portal setup (#1359)
## [0.3.45] - 2025-02-05

### 🐛 Bug Fixes

- *(reporting)* Correct JSON path in sumsub extractor postgres query (#1356)
- *(reporting)* Rename env vars for tap-sumsubapi to match lana (#1358)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.45-dev [ci skip]
- *(reporting)* Persona.xml report migrated to dbt (#1345)
## [0.3.44] - 2025-02-04

### 🚀 Features

- Kratos user authentication id (#1354)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.44-dev [ci skip]
## [0.3.43] - 2025-02-04

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.43-dev [ci skip]
- Remove app/customer (#1352)
## [0.3.42] - 2025-02-04

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.42-dev [ci skip]
- Padding in table rows (#1313)
## [0.3.41] - 2025-02-04

### 🚀 Features

- Customer server (#1333)

### 🐛 Bug Fixes

- Weird layout in auth guard (#1350)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.40-dev [ci skip]
- Client via memo to avoid recreating apollo client (#1349)
- Bump cala (#1346)
## [0.3.40] - 2025-02-04

### 🚀 Features

- *(reporting)* Meltano custom extractor to sync applicant data from sumsub (#1334)

### 🐛 Bug Fixes

- Bats e2e ci test (#1348)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.39-dev [ci skip]
- Remove statements db-op commits (#1342)
- Bump cala (#1343)
- Fix dockerfile path in release
## [0.3.38] - 2025-02-03

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.37-dev [ci skip]
- Nextjs bundle build with env values (#1341)
## [0.3.37] - 2025-02-03

### 🐛 Bug Fixes

- Admin-panel dockerfile

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.34-dev [ci skip]
- Remove cala-setup (#1340)
## [0.3.35] - 2025-02-03

### 🐛 Bug Fixes

- Admin-panel docker build

### 🚜 Refactor

- Move customer to core module (#1339)
## [0.3.33] - 2025-02-03

### 🚜 Refactor

- Move components to seprate package (#1336)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.33-dev [ci skip]
## [0.3.32] - 2025-01-31

### 🚀 Features

- Admin auth through kratos (#1278)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.32-dev [ci skip]
## [0.3.31] - 2025-01-30

### 🚀 Features

- Credit facility repayment plan (#1329)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.31-dev [ci skip]
## [0.3.30] - 2025-01-30

### 🚀 Features

- *(reporting)* Dbt credit facility models (#1328)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.30-dev [ci skip]
## [0.3.29] - 2025-01-30

### 🚀 Features

- Bootstrap (#1291)

### 🧪 Testing

- *(cypress)* Fix browser stack resolution (#1315)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.29-dev [ci skip]
## [0.3.28] - 2025-01-27

### 🐛 Bug Fixes

- *(reporting)* Replace xml special characters (#1325)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.28-dev [ci skip]
## [0.3.27] - 2025-01-27

### 🚀 Features

- Add balance sheet implementation (#1319)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.27-dev [ci skip]
## [0.3.26] - 2025-01-27

### 🐛 Bug Fixes

- Withdraw dialog (#1320)
- *(reporting)* Remove balance sheet account set refernece in saldo_cuenta.xml (#1324)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.25-dev [ci skip]
## [0.3.25] - 2025-01-27

### ⚙️ Miscellaneous Tasks

- Run postgres to bigquery dbt every 5 mins (#1321)
## [0.3.24] - 2025-01-27

### 🐛 Bug Fixes

- Deposit cache (#1318)

### 🚜 Refactor

- Rename dataform_output -> dbt_output for reports (#1323)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.23-dev [ci skip]
## [0.3.23] - 2025-01-27

### 🐛 Bug Fixes

- Dbt tvm udfs (#1317)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.22-dev [ci skip]
## [0.3.21] - 2025-01-27

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.21-dev [ci skip]
- *(reporting)* Referencia.xml dbt (#1307)
## [0.3.20] - 2025-01-27

### 🚀 Features

- Dbt tvm udfs (#1312)
- Add back embedded statements (#1309)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.19-dev [ci skip]
- *(reporting)* Materialize regulatory reports (#1306)
- Remove dataform from reports (#1316)
## [0.3.19] - 2025-01-27

### ⚙️ Miscellaneous Tasks

- Query dbt reports instead of dataform reports (#1300)
## [0.3.18] - 2025-01-26

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.18-dev [ci skip]
- *(reporting)* Referencia_garantia.xml report migrated to dbt (#1299)
## [0.3.17] - 2025-01-26

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.17-dev [ci skip]
- Output dbt dataset name (#1302)
- *(reporting)* Materialize raw credit facility events (#1303)
## [0.3.16] - 2025-01-26

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.16-dev [ci skip]
- *(reporting)* Use dbt instead of dataform dataset for holistics (#1301)
## [0.3.15] - 2025-01-26

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.15-dev [ci skip]
- *(reporting)* Schedule dbt to run after extracting and loading data (#1298)
## [0.3.14] - 2025-01-26

### 🚀 Features

- *(reporting)* Custom extractor for bitfinex API (#1294)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.14-dev [ci skip]
## [0.3.13] - 2025-01-24

### 🚀 Features

- Add airflow orchestrator (#1296)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.13-dev [ci skip]
- Fix meltano img path in bump script (#1297)
## [0.3.12] - 2025-01-24

### 🐛 Bug Fixes

- Meltano alias in flake (#1295)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.12-dev [ci skip]
## [0.3.11] - 2025-01-23

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.10-dev [ci skip]
- Use repo-dev-out to supress release loop
## [0.3.10] - 2025-01-23

### ⚙️ Miscellaneous Tasks

- Remove slack
- Dummy commit for ci
## [0.3.9] - 2025-01-23

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.9-dev
- Skip set-dev-version commits
## [0.3.8] - 2025-01-22

### 🚀 Features

- *(reporting)* Initial dbt migration (#1276)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.8-dev
## [0.3.7] - 2025-01-22

### 🚀 Features

- Disbursal velocity checks (#1268)

### 🐛 Bug Fixes

- *(reporting)* Use same location for raw dataset and dbt dataset in dev (#1286)
- Remove cala and tf from tiltfile (#1290)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.7-dev
- Remove cala-enterprise (#1273)
- *(dev)* Fix meltano alias (#1288)
- Add meltano img tagging to release job (#1287)
- Fix meltano img digest path
- Remove 'add_transaction_account' from charts (#1279)
## [0.3.6] - 2025-01-20

### 🚀 Features

- Add chart entity projection (#1258)

### 🐛 Bug Fixes

- Data-table loading state and keyboard controls (#1274)

### ⚙️ Miscellaneous Tasks

- Adding footer actions to command Palette
- Use imported react state
- *(dev)* Set version to 0.3.6-dev
- Meltano docker image (#1277)
- Remove chart transaction account (#1266)
## [0.3.5] - 2025-01-17

### 🚀 Features

- Meltano (#1269)

### 🐛 Bug Fixes

- *(reporting)* Fix some minor bugs and failing assertions (#1265)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.5-dev
## [0.3.4] - 2025-01-16

### 🐛 Bug Fixes

- Cache flicker (#1267)

### 🚜 Refactor

- Change chart 'code' abstraction to 'path' (#1248)
- Use account id from deposits instead of customers (#1239)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.4-dev
## [0.3.3] - 2025-01-16

### 🚀 Features

- Implement command menu for keyboard navigation (#1259)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.3-dev
## [0.3.2] - 2025-01-15

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.3.2-dev
## [0.3.1] - 2025-01-15

### 🚀 Features

- *(reporting)* Link ledger accounts to customers and credit facilities (#1260)
- Overdraft prevention (#1253)

### 🐛 Bug Fixes

- Cypress tests (#1254)

### ⚙️ Miscellaneous Tasks

- Create btn on credit facility needs explanatory hover (#1252)
- *(dev)* Set version to 0.3.1-dev
## [0.3.0] - 2025-01-14

### 🚀 Features

- [**breaking**] Embed ledger (#1146)

### 🐛 Bug Fixes

- Use core deposit action and object for authorization (#1250)

### ⚙️ Miscellaneous Tasks

- Use resetForm when closing dialog
- *(dev)* Set version to 0.2.80-dev
- Remove customer callback (#1238)
- Default to ZERO balance (#1249)
- *(reporting)* Add convenience facts to make running sums on holistics easier (#1251)
## [0.2.79] - 2025-01-13

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.79-dev
## [0.2.78] - 2025-01-13

### ⚙️ Miscellaneous Tasks

- Pass appVersion variable from server component (#1236)
- Increase navigation text size for mobile devices
- *(dev)* Set version to 0.2.78-dev
## [0.2.77] - 2025-01-11

### ⚙️ Miscellaneous Tasks

- Use client for app_version (#1237)
- Remove customer selection list from create btn (#1241)
- *(dev)* Set version to 0.2.77-dev
## [0.2.76] - 2025-01-09

### ⚙️ Miscellaneous Tasks

- Prevent page refresh while polling (#1233)
- Install netlify cli in manual action
- *(dev)* Set version to 0.2.76-dev
## [0.2.75] - 2025-01-09

### 🐛 Bug Fixes

- *(reporting)* Use account_key instead of internal id in account sets output table (#1232)
- Handle zero disbursed amount for check_disbursal function (#1229)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.75-dev
## [0.2.74] - 2025-01-08

### 🐛 Bug Fixes

- *(reporting)* Use project ID not project name in dataform config (#1230)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.74-dev
## [0.2.73] - 2025-01-08

### 🚀 Features

- Show version number in admin-panel (#1226)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.73-dev
- Rename staging env to galoy-staging (#1227)
## [0.2.72] - 2025-01-08

### ⚙️ Miscellaneous Tasks

- Contextual create (#1223)
- *(dev)* Set version to 0.2.72-dev
- *(reporting)* A hack to execute star-schame tables (#1225)
## [0.2.71] - 2025-01-07

### 🐛 Bug Fixes

- Admin panel docs upload

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.71-dev
## [0.2.70] - 2025-01-07

### 🐛 Bug Fixes

- Regulatory-reporting responsive view

### ⚙️ Miscellaneous Tasks

- Set default fetchPolicy cache-and-network (#1217)
- *(dev)* Set version to 0.2.70-dev
- Adding terraform fmt checker in ci (#1204)
- Automated manuals (#1216)
- Bump flake (#1222)
## [0.2.69] - 2025-01-05

### 🐛 Bug Fixes

- Dialog and scroll (#1213)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.69-dev
- BFX_LOCAL_PRICE (#1165)
## [0.2.68] - 2025-01-03

### 🐛 Bug Fixes

- Data-table navigation
- Storybooks (#1209)
- Add report_events to bq setup (#1210)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.68-dev
## [0.2.67] - 2025-01-01

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.67-dev
## [0.2.66] - 2025-01-01

### ⚙️ Miscellaneous Tasks

- Responsive admin-panel (#1208)
- *(dev)* Set version to 0.2.66-dev
## [0.2.65] - 2024-12-31

### 🚀 Features

- Add copy button, loading state and text truncation to KYC link

### 🐛 Bug Fixes

- Capital in nav
- Admin-panel lint
- *(reporting)* Approved event to approval_process_completed (#1206)

### 📚 Documentation

- Updating bq setup (#1207)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.65-dev
- Fmt (#1203)
- *(reporting)* Remove redundant CTEs (#1192)
- *(reporting)* Remove redundant singleton dimension tables (#1193)
- *(reporting)* Configure dataform to execute daily (#1205)
## [0.2.64] - 2024-12-30

### 🐛 Bug Fixes

- Use correct service account in run-on-nix-host.sh (#1200)

### ⚙️ Miscellaneous Tasks

- Cache changes in credit facility (#1199)
- Using "enter" from keyboard should confirm user creation (#1201)
- *(dev)* Set version to 0.2.64-dev
## [0.2.63] - 2024-12-28

### ⚙️ Miscellaneous Tasks

- Remove cache on creation
- *(dev)* Set version to 0.2.63-dev
- Remove lana bank chart (#1197)
- Use galoy's concourse instead on blink's (#1198)
## [0.2.62] - 2024-12-27

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.62-dev
## [0.2.61] - 2024-12-27

### 🐛 Bug Fixes

- Spelling – commities --> committees

### ⚙️ Miscellaneous Tasks

- Use correct git_charts_branch value in ci/values.yml (#1189)
- *(dev)* Set version to 0.2.60-dev
- Separate pages for credit facility tabs (#1096)
- Add polling for facility details
- One time fee rate -> structuring fee rate
- *(fix)* Fix invalid references (#1194)
## [0.2.59] - 2024-12-26

### 🐛 Bug Fixes

- Admin-panel caching (#1188)

### ⚙️ Miscellaneous Tasks

- Switch concourse instance (#1184)
## [0.2.60] - 2024-12-20

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.58-dev
## [0.2.57] - 2024-12-20

### 🐛 Bug Fixes

- Pdf sequence
- Credit facility facts & dims data types (#1164)
- *(reporting)* Grant holistics SA metaViewer for whole project (#1174)

### 📚 Documentation

- Docs on tilt up

### ⚙️ Miscellaneous Tasks

- Use formated number for input (#1155)
- Misc chagnes in manuals (#1156)
- Improve manuals (#1157)
- Link table of content with pages
- *(dev)* Set version to 0.2.57-dev
- Reliable tilt up and remove backend_env (#1158)
- Customer manual with table of content (#1159)
- Table of content in transactions manuals (#1160)
- Table of content in users manual (#1162)
- Remove zoomed-in screenshot from report manual
- Update credit facilities manual (#1161)
- Cypress kyc tests (#1170)
## [0.2.56] - 2024-12-16

### 🚀 Features

- Credit facility star schema (#1137)

### 🧪 Testing

- Financials cypress (#1143)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.56-dev
- *(reporting)* Use int64 primary keys in outputs (#1141)
## [0.2.55] - 2024-12-15

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.55-dev
## [0.2.54] - 2024-12-13

### 🚀 Features

- Dimension table for account set hierarchy (#1133)
- Mac duration date (#1135)
- Add facility structuring fee (#1079)

### 🐛 Bug Fixes

- Clippy
- Terms template and facility tests (#1129)
- Missing 'oneTimeFeeRate' field in cypress test (#1140)
- Grant holistics sa metadataViewer role (#1138)

### 🧪 Testing

- Governance and users test (#1134)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.54-dev
## [0.2.53] - 2024-12-10

### 🚀 Features

- Configure dataform invocation to only execute report queries (#1127)

### 🐛 Bug Fixes

- Re-add package-lock.json (#1131)

### ⚙️ Miscellaneous Tasks

- *(admin-panel)* Logo on the bottom (#1125)
- *(dev)* Set version to 0.2.53-dev
- Bump dataform cli (#1100)
- Add terraform configuration for holistics (#1130)
- Tf fmt
## [0.2.52] - 2024-12-09

### 🚀 Features

- Credit facility tvm (#1071)

### 🚜 Refactor

- Separate interest incurrence/accrual jobs (#1045)
- Simplify disbursal record functions (#1099)

### ⚙️ Miscellaneous Tasks

- *(admin-panel)* Use view btn in Transactions
- *(dev)* Set version to 0.2.52-dev
- Remove time display from list page dates (#1120)

### ◀️ Revert

- Reduce credit facility months (#1122)
## [0.2.51] - 2024-12-06

### 🐛 Bug Fixes

- *(admin-panel)* Border in loading state table

### ⚙️ Miscellaneous Tasks

- *(admin-panel)* Misc changes
- *(dev)* Set version to 0.2.51-dev
## [0.2.50] - 2024-12-06

### ⚙️ Miscellaneous Tasks

- Use btn in table to navigate (#1112)
- Storybooks loading state (#1110)
- Update sidebar groups (#1105)
- Max_concurrency for jobs (#1056)
- *(dev)* Set version to 0.2.50-dev
## [0.2.49] - 2024-12-05

### 🚜 Refactor

- Cala es entity (#1097)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.49-dev
- Adding License to repo (#1085)
- Breadcrumb on header (#1109)
- Disable auto-report & entity-changed exports (#1083)
- Stop skipping z-report.bats (#1113)
## [0.2.48] - 2024-12-04

### 🚀 Features

- Add facility activation process manager (#1070)

### 🐛 Bug Fixes

- Login screen layout (#1094)
- Dashboard flicker (#1101)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.47-dev
- Customer tabs in separate route (#1084)
- Update rust & nix (#1095)
## [0.2.47] - 2024-12-02

### ⚙️ Miscellaneous Tasks

- *(es-entity)* Batch create (#1082)
## [0.2.46] - 2024-11-29

### 🚀 Features

- *(reporting)* Initial star-schema model tables (#1046)

### 🐛 Bug Fixes

- Move ledger operation for disbursal concurrent retries (#1053)
- Dataform lava-dev to lana-dev (#1059)
- *(admin-panel)* Header padding
- Remove redundant update op (#1063)
- *(es-entity)* Derive cursor name from entity

### 🚜 Refactor

- Move ui from nested index.tsx to root-level named files
- *(es-entity)* Derive cursor name from entity
- Record collateral update before ledger call (#1057)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.46-dev
- Cypress headless
- Customer page storybook (#1042)
- Es-entity ignore-prefix (#1048)
- Use faker js and separate details (#1050)
- Add storybook publish to netlify (#1052)
- Mock data setup for storybooks (#1051)
- Storybook on main and pr on apps (#1061)
- Trigger for storybook - initial
- Re-apply lana-dev (#1058)
- Use shadcn sidebar (#1060)
- Small DbOp fns
- User details and withdraw storybook (#1064)
- *(es-entity)* Add find_all_in_tx
- Storybooks for financials (#1067)
- Audit-log list page (#1065)
- Check new cvl before initiating disbursal (#1003)
- Dashboard card storybook (#1073)
- Better data for terms-template storybook (#1072)
- Dedicated bot comment for storybooks (#1068)
- Navbar in pages storybook (#1078)
## [0.2.45] - 2024-11-25

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.45-dev
## [0.2.44] - 2024-11-25

### 🚜 Refactor

- Remove redundant approval process page (#1037)
- Rename to lana in tf setup (#1043)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.44-dev
- Storybook setup and misc refactoring (#1038)
- Remove storybook from shadcn component
- Rename lava -> lana (#1039)
- Story books for table components (#1041)
- Use shadcn select component (#1044)
- Storybooks from details component (#1040)
## [0.2.43] - 2024-11-23

### 🐛 Bug Fixes

- Customer email
- Z-report.bats should not depend on the exact number of reports (#1033)

### ⚙️ Miscellaneous Tasks

- Remove redundant view column
- *(dev)* Set version to 0.2.43-dev
- *(reporting)* Update sqlx names to match new directory structure (#1025)
- Trying sim-time for tests (#1011)
- Lava -> lana renaming (#1036)
## [0.2.42] - 2024-11-22

### ⚙️ Miscellaneous Tasks

- Misc refactoring (#1032)
- *(dev)* Set version to 0.2.42-dev
## [0.2.41] - 2024-11-22

### 🚜 Refactor

- Use new find_many function in entity repos (#1023)

### ⚙️ Miscellaneous Tasks

- Moving primitives to ui (#1026)
- *(dev)* Set version to 0.2.41-dev
- Remove motion and material tailwind lib (#1028)
- Using details card for user page (#1029)
- Separate apps check-code (#1027)
## [0.2.40] - 2024-11-21

### 🚜 Refactor

- Details Page (#1024)

### ⚙️ Miscellaneous Tasks

- Rename customer page heading
- *(dev)* Set version to 0.2.40-dev
## [0.2.39] - 2024-11-20

### 🚀 Features

- Pdfs with correct ss

### 🐛 Bug Fixes

- Paginated table filter all effect

### 🚜 Refactor

- *(reporting)* Reorganize dataform files in definitions (#1022)

### ⚙️ Miscellaneous Tasks

- Improve load on list page (#1012)
- Close modal on navigation (#1014)
- *(dev)* Set version to 0.2.39-dev
- *(es-entity)* Add find_many with generic filter and cursor (#1015)
- Adding types to currencies (#1018)
## [0.2.38] - 2024-11-14

### ⚙️ Miscellaneous Tasks

- Add check-item in paginated table filters (#1010)
- *(dev)* Set version to 0.2.37-dev
## [0.2.37] - 2024-11-14

### ⚙️ Miscellaneous Tasks

- Remove frontend sorts (#1008)
- *(dev)* Set version to 0.2.36-dev
## [0.2.35] - 2024-11-13

### 🚀 Features

- Add filters to Customers query (#996)
- Regulatory user manuals (#1007)

### 🐛 Bug Fixes

- Consistent ordering for user roles (#1002)
- Cypress run for bstack

### 🚜 Refactor

- Nested credit facility accrual (#1001)

### 🧪 Testing

- Facility and terms template creation (#1004)
- Facility e2e (#1009)

### ⚙️ Miscellaneous Tasks

- Consistent no data message for tables
- *(dev)* Set version to 0.2.35-dev
- Add breadcrumb on list page (#1005)
- Don't remove test ids
## [0.2.34] - 2024-11-13

### 🐛 Bug Fixes

- Cypress local (#1000)

### ⚙️ Miscellaneous Tasks

- Browserstack setup (#994)
- *(dev)* Set version to 0.2.34-dev
## [0.2.33] - 2024-11-12

### 🐛 Bug Fixes

- Customer balance

### 🚜 Refactor

- Move list filters from separate use-cases to arg (#985)

### 🧪 Testing

- Add log for flaky step (#995)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.33-dev
## [0.2.32] - 2024-11-12

### 🐛 Bug Fixes

- Some cache busting and undef errors

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.32-dev
- Setup cypress and auth (#993)
## [0.2.31] - 2024-11-12

### 🐛 Bug Fixes

- Small ui items (#983)

### 🚜 Refactor

- Resolve balances from entity instead of ledger (#990)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.31-dev
- Adding global loading state (#988)
- Remove loading component
- Sim-time (#989)
## [0.2.30] - 2024-11-11

### ⚙️ Miscellaneous Tasks

- Table component for non paginated (#982)
- *(dev)* Set version to 0.2.30-dev
## [0.2.29] - 2024-11-11

### 🚀 Features

- Final report table for saldo_cuenta.xml from norm NPB4-16 (#956)

### 🚜 Refactor

- Improve `*_errors` queries  (#958)

### ⚙️ Miscellaneous Tasks

- Add pendingFacilities with space btw
- *(dev)* Set version to 0.2.29-dev
- Change list_reports order
## [0.2.28] - 2024-11-08

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.28-dev
## [0.2.27] - 2024-11-08

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.27-dev
- Remove Pending Facilities
- Add tracing to approve_disbursal
## [0.2.26] - 2024-11-08

### 🐛 Bug Fixes

- Admin-panel lint

### 🚜 Refactor

- Move disbursal confirm to approval process manager (#980)

### ⚙️ Miscellaneous Tasks

- Misc changes in navbar
- *(dev)* Set version to 0.2.26-dev
## [0.2.25] - 2024-11-08

### ⚙️ Miscellaneous Tasks

- *(ui)* Random small css fixes
- *(dev)* Set version to 0.2.25-dev
## [0.2.24] - 2024-11-07

### ⚙️ Miscellaneous Tasks

- Sort direction in ui
- Filter (#979)
- *(dev)* Set version to 0.2.24-dev
## [0.2.23] - 2024-11-07

### 🚀 Features

- *(ui)* Create entity from entity listing page (#974)
- Add facility filter-by queries (#976)
- Sort to uis
- Add direction option to sort in gql (#978)

### 🐛 Bug Fixes

- Apollo client (#977)

### 🚜 Refactor

- Unify customers queries with new sort param (#972)
- DisbursalExecuted

### ⚙️ Miscellaneous Tasks

- Disbursal details page (#975)
- Add ui links to tiltfile resources (#967)
- *(dev)* Set version to 0.2.23-dev
## [0.2.22] - 2024-11-07

### 🚀 Features

- *(ui)* Remove search (#971)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.22-dev
## [0.2.21] - 2024-11-07

### ⚙️ Miscellaneous Tasks

- Remove credit facility complete from front-end
- *(dev)* Set version to 0.2.21-dev
- Disbursal query
## [0.2.20] - 2024-11-07

### 🐛 Bug Fixes

- Misc
- Active facilities fonts (#968)
- Dashboard (#969)
- Check-code (#970)

### 🚜 Refactor

- *(outbox)* Rename persist -> publish_persisted

### ⚙️ Miscellaneous Tasks

- Added dashboard query data (#966)
- *(dev)* Set version to 0.2.19-dev
- Expose credit_facility from disbursal (#964)
- Extract out money module (#965)
## [0.2.19] - 2024-11-07

### ⚙️ Miscellaneous Tasks

- Use Skeleton for loading (#963)
- *(dev)* Set version to 0.2.18-dev
- Expose total_disbursed (#961)
## [0.2.17] - 2024-11-07

### 🐛 Bug Fixes

- More fixes (#960)

### ⚙️ Miscellaneous Tasks

- Consistent list and misc ui changes (#962)
- *(dev)* Set version to 0.2.17-dev
## [0.2.16] - 2024-11-06

### 🚀 Features

- Input numeric comma (#959)

### ⚙️ Miscellaneous Tasks

- Proper lists (#957)
- *(dev)* Set version to 0.2.16-dev
## [0.2.15] - 2024-11-06

### 🚀 Features

- Context aware create (#949)
- Fix create button and misc ui stuff (#955)
- Add 'creditFacilitiesByCvl' query (#954)

### ⚙️ Miscellaneous Tasks

- Changes in details page (#951)
- *(dev)* Set version to 0.2.15-dev
## [0.2.14] - 2024-11-06

### 🚀 Features

- Sort connections by field in grapqhl (#945)
- Generate saldo_cuenta.xml according to NPB4-16 (#932)

### 🐛 Bug Fixes

- Schema.graphql

### 🚜 Refactor

- Use old codebase for faster iteration - staging working with new ui (#940)

### ⚙️ Miscellaneous Tasks

- Reexport mtw from dedicated file (#938)
- *(dev)* Set version to 0.2.14-dev
- Ui changes (#942)
- Basic dashboard (#934)
- More ui changes (#946)
- Customer paginated list (#941)
- Logic for active facilities (#944)
- Add customer.status column (#950)
## [0.2.13] - 2024-11-05

### ⚙️ Miscellaneous Tasks

- Change next.js version (#936)
- *(dev)* Set version to 0.2.13-dev
## [0.2.12] - 2024-11-05

### ⚙️ Miscellaneous Tasks

- *(ui)* Api health (#935)
- *(dev)* Set version to 0.2.12-dev
## [0.2.11] - 2024-11-05

### 🚜 Refactor

- Call authz.audit() (#926)
- Add begin fn to EsRepo (#930)
- Disbursement -> disbursal (#928)
- Hot lava fresh design (#931)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.11-dev
- Withdraw -> withdrawal (#924)
- Remove loans  (#927)
## [0.2.10] - 2024-11-02

### 🚜 Refactor

- Remove core/shared_primitives (#909)
- Change how EsEntityError is handled (#913)

### ⚙️ Miscellaneous Tasks

- Nb ui comments (#910)
- Identify concurrent modification (#911)
- Unused files (#919)
- Enable force processing on read (#918)
- Moving report to es-entity (#907)
- Remove old entity dir (#920)
- Make retry more resiliant for CreditFacility (#921)
- Help ensure idempotency with #[must_use] (#922)
## [0.2.9] - 2024-11-01

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.9-dev
## [0.2.8] - 2024-11-01

### 🐛 Bug Fixes

- Apps linting
- Add disbursement target (#908)

### ⚙️ Miscellaneous Tasks

- Fix more test lints
- *(dev)* Set version to 0.2.8-dev
- Disbursement approval ui (#906)
## [0.2.7] - 2024-10-31

### 🚜 Refactor

- -> canSubmitDecision

### ⚙️ Miscellaneous Tasks

- Set tracing sampler to AlwaysOn
- Add bacon
- *(dev)* Set version to 0.2.7-dev
- Revert default bacon config
- Remove unused code
- Fix some linting
- Add health check to server
## [0.2.6] - 2024-10-31

### 🚀 Features

- Loan treasury projections (#832)

### 🚜 Refactor

- Add approval process to disbursement (#904)
- Seperate admin server (#901)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.6-dev
- Adding approval and deny dialog (#902)
## [0.2.5] - 2024-10-30

### 🐛 Bug Fixes

- *(release)* Lava-cli in Dockerfile.relaese

### 🚜 Refactor

- Add facility activation helper with accrual job (#903)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.5-dev
## [0.2.4] - 2024-10-30

### 🐛 Bug Fixes

- Clippy

### 🚜 Refactor

- Simplify Subject system nodes (#891)

### 🧪 Testing

- Unit test facility activation_data (#893)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.4-dev
- Support order direction in es-entity (#892)
- Add governance to credit facility (#878)
- Expose ApprovalProcessStatus (#894)
- ApprovalProcessType / ApprovalProcessTarget in gql (#897)
- *(es-entity)* Bubble deserialization error (#898)
- *(gql)* ApprovalProcess.can_vote (#899)
- *(gql)* ApprovalProcessVoter.voted_at (#900)
## [0.2.3] - 2024-10-29

### 🐛 Bug Fixes

- Role enum in gel (#890)

### 🚜 Refactor

- Extract user (#876)
- Make job init typesafe (#889)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.3-dev
- *(admin-paneld)* Approval-process (#885)
## [0.2.2] - 2024-10-29

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.2-dev
## [0.2.1] - 2024-10-29

### 🐛 Bug Fixes

- Use set to check if member has been added or not (#879)
- Bug in Dockerfile.release

### 🚜 Refactor

- Simplify creation of global jobs (#877)
- Extract server (#882)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.2.1-dev
- Add approval rules union type (#880)
- Process approval mutation (#883)
- *(admin-panel)* Policy page (#884)
## [0.2.0] - 2024-10-28

### 🚀 Features

- Cvl is being shared from backend (#818)
- Allocate disbursement payments only after expiry (#830)

### 🚜 Refactor

- Use es-entity in User (#806)
- Use es-entity in document (#804)
- Credit facility es entity (#836)
- Remove core query module
- Expand facility status variants (#834)
- Move core -> core/app (#838)
- Extract audit module (#842)
- Audit / auth (#849)
- Extract shared-primitives (#850)
- Extract rbac-types crate (#851)
- Move rbac-types / app to lava (#852)
- Use es-entity in lib/job (#856)
- Indicate DuplicateUniqueJobType in job (#859)
- Use record_system_entry (#864)
- Extract cli (#875)

### 🧪 Testing

- Add failing test for accrual interest (#823)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.143-dev
- Use generated list_for_xxx (#821)
- Differentiate between incurrence_interval and accrual_interval in reporting (#831)
- Implement facility 'interest_accrued' (#829)
- Bump flake
- Loan entity es macro (#833)
- *(shared)* Bump vendored ci files (#773)
- Fix for core -> app
- Do not report completed assets (#839)
- Add core/authz (#843)
- Add lib/outbox (#854)
- Add (unused) governance module to app (#853)
- Remove core/app/src/governance
- Governance domain (#855)
- Remove redundant files
- Outbox in governance boilerplate (#857)
- Add withdraw/approval_job (#858)
- Approval rules (#861)
- Add process_type to ApprovalProcessConcluded (#863)
- Add gql for committee (#860)
- WithdrawalStatus.Initiated -> WithdrawalStatus.PendingApproval (#866)
- ApprovalProcess gql (#872)
- Es entity gql feature (#873)
- Gql for policy (#865)
## [0.1.142] - 2024-10-22

### 🚀 Features

- Add some terms-related unit tests (#771)
- Requlatory reporting for credit facilities (#815)

### 🐛 Bug Fixes

- Amount in confirm withdraw (#817)

### 🚜 Refactor

- Some es-event improvements (#793)
- Move es-entity-derive -> es-entity-macros
- Use es-entity in Withdraw (#807)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.142-dev
- Support list_by_created_at (#812)
- Deposit using es framework (#811)
## [0.1.141] - 2024-10-18

### 🚀 Features

- Implement credit facility interest job (#784)

### 🐛 Bug Fixes

- Updating the deleted flag for documents (#787)

### ⚙️ Miscellaneous Tasks

- Rename customer to customer email and remove withdrawal column (#770)
- *(dev)* Set version to 0.1.141-dev
## [0.1.140] - 2024-10-18

### ⚙️ Miscellaneous Tasks

- Remove collateralization fn (#794)
- *(dev)* Set version to 0.1.140-dev
## [0.1.139] - 2024-10-18

### ⚙️ Miscellaneous Tasks

- Add more balance details in credit facility (#791)
- Update terms template creation layout (#792)
- *(dev)* Set version to 0.1.139-dev
## [0.1.138] - 2024-10-18

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.138-dev
- Credit facility cvl job (#785)
## [0.1.137] - 2024-10-18

### 🚀 Features

- Deleting documents attached to customer (#745)

### 📚 Documentation

- Make init-bq flow more explicit (#775)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.137-dev
- Rename interval to accrualInterval (#783)
## [0.1.136] - 2024-10-17

### 🚀 Features

- Add facility remaining field to credit facility balance (#736)
- Sumsub kyc questions (#755)
- Add accrual-related ledger operations (#764)
- Add incurrence interval to terms (#762)
- Setup accrual entity (#765)

### 🐛 Bug Fixes

- Lock all_jobs when processing fail_job
- Sqlx-prepare
- Clippy
- *(admin-panel)* Readable numbers (#746)
- Pass JobId to Job::new function (#751)

### 🚜 Refactor

- Extract job into lib (#720)
- Remove valueComponent and labelComponent (#754)
- Start InterestPeriod next period in 1 second (#759)

### ⚙️ Miscellaneous Tasks

- Show errors in docs upload (#742)
- Changes in create loan modal layout (#743)
- *(dev)* Set version to 0.1.136-dev
- Bump nix deps
- Make n_attempts: Option<u32>
- *(admin-panel)* Create term template if not present (#747)
- Remove clone from events (#749)
- Remove unused action (#769)
- Add validation in terms value (#758)
## [0.1.135] - 2024-10-11

### 🐛 Bug Fixes

- Schema typo (#729)

### ⚙️ Miscellaneous Tasks

- Rename snapshot to overview (#725)
- *(dev)* Set version to 0.1.135-dev
- Credit Facility Redesign edit screen to two-column layout (#703)
## [0.1.134] - 2024-10-11

### ⚙️ Miscellaneous Tasks

- Display role in user profile (#724)
- *(dev)* Set version to 0.1.134-dev
## [0.1.133] - 2024-10-11

### 🐛 Bug Fixes

- Collateral required on creation (#721)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.133-dev
- Display details credit-facilities approve modal (#722)
- Credit facility permissions for bank manager (#723)
## [0.1.132] - 2024-10-11

### 🐛 Bug Fixes

- Typo loan -> Disbursement (#719)

### ⚙️ Miscellaneous Tasks

- Add disbursement approvers details (#718)
- *(dev)* Set version to 0.1.132-dev
## [0.1.131] - 2024-10-11

### ⚙️ Miscellaneous Tasks

- Price details in credit facility (#712)
- Misc ui improvements (#715)
- *(dev)* Set version to 0.1.131-dev
- Disbursement history (#716)
## [0.1.130] - 2024-10-11

### 🐛 Bug Fixes

- Give accountant permission to list document

### 🚜 Refactor

- Country table to include 3 digit iso code (#702)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.130-dev
- Minor additions to sumsub dataform transformations (#713)
- Disbursement Approvals (#714)
## [0.1.129] - 2024-10-10

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.129-dev
- Add transactions (#701)
## [0.1.128] - 2024-10-10

### 🚀 Features

- Initial work on integrating sumsub data in to dataform pipeline (#699)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.128-dev
- Add can_be_completed field (#700)
## [0.1.127] - 2024-10-09

### 🚀 Features

- Documents have their ui (#696)

### 🚜 Refactor

- Referencia.xml errors (#689)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.127-dev
- Use `report_` table as input to `_error` table (#688)
- Expose disbursements (#693)
- Expose User for Credit Facility Approval (#694)
- *(admin-panel)* Credit facility complete, payment mutation and details tab (#691)
## [0.1.126] - 2024-10-09

### 🚀 Features

- Credit facility complete mutation (#681)
- *(admin-panel)* Credit Facility (#680)

### 🐛 Bug Fixes

- Add missing fields (#682)

### 🚜 Refactor

- Reg report error (#683)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.126-dev
- Expose credit_facilities actionable items (#684)
- Add missing fields in credit facility gql object (#690)
## [0.1.125] - 2024-10-08

### 🚀 Features

- Add facility cvl and collateralization (#636)
- Expose authz permitted actions to gql (#669)
- *(ui)* Actionable items on the admin-panel (#670)
- Add creditFacilityPartialPayment gql operation (#663)
- Add pdf upload (#623)

### 🚜 Refactor

- Use inspect/enforce naming for authz (#661)

### 🧪 Testing

- Skip documents test in concourse

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.125-dev
- Misc improvments (#664)
- Sumsub callback to return 200 when sandbox mode is true (#667)
- Add make target to run dataform with staging data (#679)
- Credit_facility list queries (#676)
## [0.1.124] - 2024-10-04

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.124-dev
- Drop-down menu on terms template (#654)
## [0.1.123] - 2024-10-04

### 🚀 Features

- Add accountant role (#614)
- Update terms template (#648)
- Possibility of authz without audit trail in app layer (#646)

### 🐛 Bug Fixes

- Vulnerability
- Table valued function reports (#643)
- Termaction (compile error) (#653)

### 🚜 Refactor

- Split terms and terms_template (#631)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.123-dev
- Add terms to credit facility (#633)
- Remove redundant CTEs (#637)
- Example refactored error table (#638)
- Change collateral update cala templates to be more generic (#642)
- Improve reports bats test (#644)
- Not allowed cursor on disabled button (#652)
## [0.1.122] - 2024-10-01

### 🚜 Refactor

- Extract storage module (#620)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.122-dev
## [0.1.121] - 2024-10-01

### 🚀 Features

- Disbursement entity in facility (#615)
- Terms templates (#617)

### 🐛 Bug Fixes

- Add dummy dependencies to prevent concurrent execution (#611)
- Revert yaml_path -> config in cli
- Table reports from table-function reports (#626)
- Historical reports v3 (#612)
- Loan bats (#630)
- Check code

### ⚙️ Miscellaneous Tasks

- Add Confirmation Step in customer create (#613)
- *(dev)* Set version to 0.1.121-dev
- Unused var (#621)
- Fix errors when running dataform through the API (#619)
- Role should be shown in a deterministic order (#627)
- Add disbursement gql operations (#628)
## [0.1.120] - 2024-09-27

### ⚙️ Miscellaneous Tasks

- Improving variable names for cli (#600)
- *(dev)* Set version to 0.1.120-dev
- Do not make download link time configurable (#610)
- Unused import (#609)
- Filling in some missing fields in loan and collateral reports (#599)
- Use price in reports (#598)
## [0.1.119] - 2024-09-26

### 🐛 Bug Fixes

- Add prod to includes/env.js

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.119-dev
## [0.1.118] - 2024-09-26

### 🧪 Testing

- Refactor maybe update tests (#580)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.118-dev
- Bump flake
- Credit facility approve  (#593)
- Rename BQ_SERVICE_ACCOUNT to SA_CREDS_BASE64 (#602)
- Unused (#595)
- Move to lava-dev project
## [0.1.117] - 2024-09-25

### 🚀 Features

- Historical reports (#561)

### 🐛 Bug Fixes

- Sumsub integration (#587)

### 🧪 Testing

- Remove e2e loan withdraw check (#585)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.117-dev
- Make init-bq idempotent (#579)
- Unused job states (#592)
- Moving sumsub test to integration test folder (#591)
- Export price to bq (#578)
- Credit facility boilerplate (#575)
## [0.1.116] - 2024-09-23

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.116-dev
## [0.1.115] - 2024-09-23

### 🐛 Bug Fixes

- Dummy data

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.115-dev
## [0.1.114] - 2024-09-23

### 🐛 Bug Fixes

- Invalid date in dummy data (#574)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.114-dev
- Add staging config to `all` in includes/envs.js (#572)
## [0.1.113] - 2024-09-20

### 🚀 Features

- Persist sumsub events (#563)

### 🐛 Bug Fixes

- Release config

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.113-dev
- Remove unused lib (#570)
## [0.1.112] - 2024-09-20

### 🐛 Bug Fixes

- Better error capture in dataform client
- Expect on to_string

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.112-dev
## [0.1.111] - 2024-09-20

### 🚀 Features

- Reporting ui (#567)
- Add daily 'create report' job (#568)

### 🐛 Bug Fixes

- Dataform_release_config_name

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.111-dev
- Bump sqlx and sqlx-adapter (#569)
- Add df outputs to bq-setup
## [0.1.110] - 2024-09-19

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.110-dev
## [0.1.109] - 2024-09-19

### 🚀 Features

- Add 'reports' query & 'created_at' field (#564)

### 🐛 Bug Fixes

- Fixup env variable (#566)
- - -> _ in dataset names

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.109-dev
## [0.1.108] - 2024-09-19

### 🐛 Bug Fixes

- Decimal inconsistent / unexpected (#559)
- $$ in Makefile

### 🚜 Refactor

- Customer id page (#562)

### ⚙️ Miscellaneous Tasks

- Rename report output tables (#554)
- Enable back button on confirm screens (#558)
- *(dev)* Set version to 0.1.108-dev
- Initial report module (#545)
- Add dummy bq creds
- Use fake SA
- Skip report bats in concourse
- Bring fake sa in via make
- Base64 -w
## [0.1.107] - 2024-09-15

### 🐛 Bug Fixes

- AccrualAt (#544)
- Loan dropdown on customer page (#543)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.107-dev
- *(dev)* Set version to 0.1.107-dev (#532)
- Unecessary deref (#549)
## [0.1.106] - 2024-09-13

### 🐛 Bug Fixes

- Chartjs registerables (#542)

### ⚙️ Miscellaneous Tasks

- Remove confirm dilaog from withdraw (#538)
- *(dev)* Set version to 0.1.106-dev
## [0.1.105] - 2024-09-13

### 🚀 Features

- Reg report export (#533)

### ⚙️ Miscellaneous Tasks

- Remove nested quotes in table descriptions (#528)
- *(dev)* Set version to 0.1.105-dev
- Fine tune object for rbac (#514)
## [0.1.104] - 2024-09-11

### 🚀 Features

- Repayment ui (#527)
- Reg report persona xml v2 (#526)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.104-dev
- Updates for referencia.xml report (#519)
- Update telegram id (#521)
## [0.1.103] - 2024-09-10

### 🧪 Testing

- Add remaining edge cases for repayment plan (#524)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.103-dev
- Repayment plan (#523)
## [0.1.102] - 2024-09-09

### 🚀 Features

- Adding telegram id (#517)

### 🐛 Bug Fixes

- Potential vulnerability on quinn-proto (#515)

### 🚜 Refactor

- Use idiomatic parse fn in Action

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.102-dev
- Simplify authz (remove assign / revoke <role>) (#513)
- Update quinn-proto for RUSTSEC-2024-0373 (#516)
## [0.1.101] - 2024-09-06

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.101-dev
- Bump async-graphql (#503)
- Cache price (#504)
## [0.1.100] - 2024-09-06

### 🚀 Features

- Loan approval ui (#502)

### 🐛 Bug Fixes

- Fmt

### ⚙️ Miscellaneous Tasks

- Bump repo-ref in charts
- *(dev)* Set version to 0.1.100-dev
- Fix bump in charts
## [0.1.99] - 2024-09-06

### ⚙️ Miscellaneous Tasks

- Bump postgres img to 16.4
- *(dev)* Set version to 0.1.99-dev
- Bump nix
- Add bq_setup variable
## [0.1.98] - 2024-09-05

### 🐛 Bug Fixes

- Admin-panel config (#497)

### 🚜 Refactor

- Tf setup (#499)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.98-dev
- Add Dataform-repos to lava setup (#484)
## [0.1.97] - 2024-09-05

### 🚀 Features

- New ui for loan details (#496)
- Multiple approvals for loan (#478)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.97-dev
## [0.1.96] - 2024-09-05

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.96-dev
## [0.1.95] - 2024-09-05

### 🚀 Features

- Add withdrawal check in initiate_withdraw function (#489)
- New ui for customer details (#493)

### 🚜 Refactor

- Remove 'encumbrance' view in frontend (#481)
- Admin panel components (#491)
- *(admin-panel)* Move entity tightly coupled to entities into app (#492)
- Use strum crate in authz serialization (#490)

### ⚙️ Miscellaneous Tasks

- Return from edit all terms loan input to only edit principal loan input (#487)
- *(dev)* Set version to 0.1.95-dev
- *(ui)* Add withdrawal and deposit references (#485)
## [0.1.94] - 2024-09-03

### ⚙️ Miscellaneous Tasks

- GradientProgressBar component (#482)
- When creating a new user, we should be able to assign new role in the same window (#483)
- *(dev)* Set version to 0.1.94-dev
## [0.1.93] - 2024-09-03

### 🚀 Features

- Initial commit of dataform transformations (#430)

### 🐛 Bug Fixes

- Https://github.com/GaloyMoney/lava-bank/issues/450 (#475)
- Misnamed principal/interest accounts (#480)

### ⚙️ Miscellaneous Tasks

- Add price to margin CVL and liquidation CVL in loan details (#449)
- *(dev)* Set version to 0.1.92-dev
- Improve time for audit list (#468)
- Admin should be able to add other admin (#470)
- Remove post confirm screens (#472)
- Switch liabilities and Equity orderin balance sheet (#473)
- Improve error management for sumsub (#469)
- Restrict approval below margin limit (#476)
## [0.1.92] - 2024-08-30

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.91-dev
## [0.1.90] - 2024-08-30

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.90-dev
- Expose sa email
## [0.1.89] - 2024-08-30

### 🐛 Bug Fixes

- Remove zeroes from approve loan (#447)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.89-dev
- Add recorded_at field in collateralization changed event (#448)
## [0.1.88] - 2024-08-30

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.88-dev
## [0.1.87] - 2024-08-30

### 🚀 Features

- Add collateralizationStateUpdate mutation (#432)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.87-dev
- Add persist fn in loan repo (#443)
- Loan history (#428)
- Update collateralization btn (#442)
- Some ui patches for price (#444)
- Change dev dataset locations to EU (#445)
## [0.1.86] - 2024-08-29

### 🐛 Bug Fixes

- Allow multiple "" deposits
- Auth list crash

### 🚜 Refactor

- Less noisy cvl job audit

### ⚙️ Miscellaneous Tasks

- More price awareness (#431)
- *(dev)* Set version to 0.1.85-dev
## [0.1.85] - 2024-08-29

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.84-dev
- Add bq-setup
## [0.1.83] - 2024-08-29

### 🚀 Features

- Realtime price and currentCVL client side in loan details and create loan (#426)

### 🐛 Bug Fixes

- Loan pagination by ratio

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.83-dev
- List by collateralization ratio (#429)
## [0.1.82] - 2024-08-29

### 🐛 Bug Fixes

- Tracing for servers

### 🚜 Refactor

- Add 'status' to loan collaterization logic (#424)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.82-dev
- Add additional loan traces
- UsdCentsPerBtc query (#423)
- Spawn CVL job (#422)
- Add audit info to CollateralizationChanged event (#414)
- Add approvedAt and expiresAt to loan (#427)
## [0.1.81] - 2024-08-28

### 🚀 Features

- Loan collateralization state (#413)

### 🐛 Bug Fixes

- Remove redundant collateralization_state accessor

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.81-dev
- Export all events to BQ (#411)
## [0.1.80] - 2024-08-28

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.80-dev
- Add calls to price service (#412)
## [0.1.79] - 2024-08-28

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.79-dev
- Misc ui changes (#408)
- Price service boilerplate (#410)
## [0.1.78] - 2024-08-27

### 🚀 Features

- Add price awareness to loan (#377)

### ⚙️ Miscellaneous Tasks

- Expose visible navigation items (#404)
- *(dev)* Set version to 0.1.78-dev
## [0.1.77] - 2024-08-27

### 🐛 Bug Fixes

- Make OBS Liabilities credit normal (#409)
- Missing project on google_bigquery_table

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.77-dev
## [0.1.76] - 2024-08-27

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.76-dev
- Data export boilerplate (#392)
## [0.1.75] - 2024-08-27

### 🐛 Bug Fixes

- Loan origination transaction type (#403)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.75-dev
## [0.1.74] - 2024-08-26

### 🐛 Bug Fixes

- Admin me query (#398)
- Clippy

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.74-dev
- Bump tracing
- Loan origination txn (#400)
## [0.1.73] - 2024-08-26

### 🚀 Features

- Ui updates to admin panel (#391)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.73-dev
## [0.1.72] - 2024-08-23

### 🐛 Bug Fixes

- Audit window crashing (#395)
- Trial balance totals Off-BalanceSheet (4 columns) (#397)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.72-dev
- Bump cala
- Add collateral transactions (#386)
## [0.1.71] - 2024-08-22

### ⚙️ Miscellaneous Tasks

- Auto refresh when deposits/withdrawals are created (#390)
- *(dev)* Set version to 0.1.71-dev
## [0.1.70] - 2024-08-22

### 🚀 Features

- Disabling apollo cache by default and keeping it for paginated queries (#388)

### ⚙️ Miscellaneous Tasks

- Remove date range selector from Chart Of Accounts (#387)
- *(dev)* Set version to 0.1.70-dev
- Add audit info (#286)
## [0.1.69] - 2024-08-21

### ⚙️ Miscellaneous Tasks

- Remove user-id from table views (#383)
- *(dev)* Set version to 0.1.69-dev
## [0.1.68] - 2024-08-21

### 🚀 Features

- Update collateral (#330)

### ⚙️ Miscellaneous Tasks

- Date-range in financials (#378)
- *(dev)* Set version to 0.1.68-dev
## [0.1.67] - 2024-08-21

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.67-dev
- Fmt
## [0.1.66] - 2024-08-20

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.66-dev
- Default setup dataset_id
## [0.1.65] - 2024-08-20

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.65-dev
- Include initial bq setup (#375)
## [0.1.64] - 2024-08-20

### ⚙️ Miscellaneous Tasks

- Bump lava-setup in private-charts
- *(dev)* Set version to 0.1.64-dev
- Fix cp
- Add ranged-balance field for all relevant queries (#326)
- Bump cala schema
- Loan payments transactions (#329)
## [0.1.63] - 2024-08-16

### 🚜 Refactor

- Extract lava-setup tf module (#373)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.63-dev
- Tmp pass audit
## [0.1.62] - 2024-08-16

### 🚀 Features

- Dedicated loan screen + deep-links for loans (#368)
- Find customer by email (#371)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.62-dev
- Ui bugs (#369)
- Remove start_at field from loan entity (#370)
## [0.1.61] - 2024-08-16

### 🚀 Features

- Add cash flow statement query (#278)
- Cancel withdraw (#357)

### 🐛 Bug Fixes

- Redirect to correct page for new user (#359)
- Handle "" references
- Correctly deserialize WithdrawAction

### 🚜 Refactor

- Change 'as_ref' to 'as_deref' (#366)

### ⚙️ Miscellaneous Tasks

- *(admin-panel)* Use btc insted of sats (#345)
- *(dev)* Set version to 0.1.61-dev
- Split loan receivables accounting (#277)
- Implement pagination for audit (#361)
- Show more details in withdraw confirm screen (#347)
## [0.1.60] - 2024-08-14

### 🚀 Features

- List loans (#322)

### 🐛 Bug Fixes

- Prohibit withdraw overdraft (#343)

### 🧪 Testing

- Uncomment assert_accounts_balanced in customer.bats (#338)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.60-dev
- Display more entries by default (#344)
## [0.1.59] - 2024-08-14

### ⚙️ Miscellaneous Tasks

- Balance sheet should show sum liabilities + equity (#320)
- Use radio buttons on financials (#341)
- *(dev)* Set version to 0.1.59-dev
## [0.1.58] - 2024-08-13

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.58-dev
## [0.1.57] - 2024-08-13

### ⚙️ Miscellaneous Tasks

- Creating an enum for Subject (#287)
- *(dev)* Set version to 0.1.57-dev
- Assign superuser role on startup
## [0.1.56] - 2024-08-13

### ⚙️ Miscellaneous Tasks

- Collateral inputs should be in BTC, not sats (#328)
- *(admin-panel)* Use netCredit for Revenue, netDebit for Expenses for pnl (#304)
- *(dev)* Set version to 0.1.56-dev
## [0.1.55] - 2024-08-13

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.55-dev
## [0.1.54] - 2024-08-13

### 🚀 Features

- *(admin-panel)* Withdraw and deposit (#317)

### 🐛 Bug Fixes

- Pagination for deposits and withdrawals (#319)

### ⚙️ Miscellaneous Tasks

- Navigate to customer id on customer creation (#324)
- Pass NEXTAUTH_SECRET explicitly so that runtime env is picked (#318)
- Change partial payment to payment (#321)
- *(dev)* Set version to 0.1.54-dev
## [0.1.53] - 2024-08-12

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.53-dev
- Left side menu - bundle chart of accounts / pnl / balance sheet (#303)
- Add deposit and withdrawal queries (#305)
- Add confirmed field for Withdrawal object (#314)
## [0.1.52] - 2024-08-12

### 🐛 Bug Fixes

- Correctly load casbin policy before enforcing (#306)

### ⚙️ Miscellaneous Tasks

- Clarification between auth and customer (#285)
- *(dev)* Set version to 0.1.52-dev
- Bump flake
## [0.1.51] - 2024-08-12

### 🚀 Features

- *(admin-panel)* Balance Sheet table (#281)

### 🚜 Refactor

- Add macros in authorization/mod.rs (#295)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.51-dev
## [0.1.50] - 2024-08-09

### 🚀 Features

- *(admin-panel)* Profit and loss statement (#276)

### 🚜 Refactor

- Remove bfx integration cont. (#273)
- Bring back withdraw use cases (#284)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.50-dev
- Add authorization for ledger (#283)
## [0.1.49] - 2024-08-07

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.49-dev
## [0.1.48] - 2024-08-07

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.48-dev
## [0.1.47] - 2024-08-07

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.46-dev
## [0.1.46] - 2024-08-07

### ⚙️ Miscellaneous Tasks

- Pagination for audit logs (#267)
## [0.1.45] - 2024-08-07

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.45-dev
## [0.1.44] - 2024-08-07

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.44-dev
## [0.1.43] - 2024-08-07

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.43-dev
## [0.1.42] - 2024-08-07

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.42-dev
## [0.1.41] - 2024-08-07

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.41-dev
## [0.1.40] - 2024-08-07

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.40-dev
## [0.1.39] - 2024-08-07

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.39-dev
## [0.1.38] - 2024-08-07

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.38-dev
## [0.1.37] - 2024-08-07

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.37-dev
## [0.1.36] - 2024-08-07

### 🧪 Testing

- Add superuser bats test (#275)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.36-dev
- Paginated query for customers page (#272)
## [0.1.35] - 2024-08-06

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.35-dev
## [0.1.34] - 2024-08-06

### 🚜 Refactor

- Remove bfx integration (#269)

### ⚙️ Miscellaneous Tasks

- Pre-load Update terms modal with the current terms (#264)
- Show percentage for CVL value (#266)
- *(dev)* Set version to 0.1.33-dev
- Add admin role  (#265)
- Use pct for annual interest (#270)
- Run integration tests in file-serial mode (#274)
## [0.1.33] - 2024-08-06

### ⚙️ Miscellaneous Tasks

- Auto-trigger release
- Bump cala schema
## [0.1.32] - 2024-08-06

### 🚀 Features

- Authz for audit (#253)

### 💼 Other

- Bump flake (#252)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.32-dev
- Combine cala account set details/with-balance cala queries (#225)
- Authorization for customer (#261)
## [0.1.31] - 2024-08-05

### 🚀 Features

- Simple audit log (#248)

### 🐛 Bug Fixes

- Oathkeeper rules (#245)

### 💼 Other

- Bump flake (#251)

### ⚙️ Miscellaneous Tasks

- Loan card ui cahnges (#243)
- *(dev)* Set version to 0.1.31-dev
- Add negative number error handling (#219)
- CustomerCreate  (#244)
- Remove mailslurper (#250)
- Disable submit utton while OTP submission is Processing (#249)
- Admin me query (#247)
## [0.1.30] - 2024-08-01

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.30-dev
## [0.1.29] - 2024-08-01

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.29-dev
- *(admin-panel)* Error message display on frontend (#233)
- Impl usecases for Users service (#222)
- Deprecate accountSet query (#227)
## [0.1.28] - 2024-07-31

### 🐛 Bug Fixes

- Account for day when loan is approved (#232)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.27-dev
- Use basepath only for development (#231)
## [0.1.27] - 2024-07-30

### 🚀 Features

- Signin using magic link (#220)

### 🐛 Bug Fixes

- Add net income to equity account set (#223)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.26-dev
- Clean up reset tf (#228)
## [0.1.25] - 2024-07-30

### 🚀 Features

- Add profit loss statement query (#210)

### ⚙️ Miscellaneous Tasks

- Add more logs (#213)
- *(dev)* Set version to 0.1.25-dev
- User -> customer everywhere (#215)
- Remove customer portal logs (#216)
- User entity (#217)
- Remove unused balance_sheet.tf file (#209)
- Remove AdminAuthContext check in public me query (#226)
## [0.1.24] - 2024-07-23

### 🚀 Features

- *(admin-panel)* Default terms (#194)
- *(customer-portal)* Current terms (#197)
- Add new balance sheet report account set and query (#205)
- On off balance sheet in admin panel ui (#211)
- Paginated fetch mores (#202)
- Authorization scaffolder (#181)

### ⚙️ Miscellaneous Tasks

- Add more logging to customer portal get me and session (#207)
- *(dev)* Set version to 0.1.24-dev
## [0.1.23] - 2024-07-22

### 🐛 Bug Fixes

- Send all headers from apollo rsc to core gql (#206)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.23-dev
- Add terms for individual loans (#199)
## [0.1.22] - 2024-07-19

### 🚜 Refactor

- Standardize account & account set types used in reports (#204)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.22-dev
## [0.1.21] - 2024-07-18

### ⚙️ Miscellaneous Tasks

- *(admin-panel)* Separate loan create and approve (#201)
- Using cookie in headers for auth (#203)
- *(dev)* Set version to 0.1.21-dev
## [0.1.20] - 2024-07-18

### 🚀 Features

- Split off-balance-sheet accounts into separate reports (#196)

### 🐛 Bug Fixes

- *(admin-panel)* Loans header (#192)

### 🚜 Refactor

- Clean up chart of accounts (#184)
- Change subAccounts field to Connection type (#180)

### ⚙️ Miscellaneous Tasks

- Log apollo request (#191)
- *(dev)* Set version to 0.1.20-dev
- Expose currentTerms query in admin and public schema (#193)
- Clean up organization of accounts (#189)
- Split loan creation and approval (#198)
- Add loan status (#200)
## [0.1.19] - 2024-07-12

### 🐛 Bug Fixes

- Cypress test for hydration exception (#188)

### ⚙️ Miscellaneous Tasks

- App layout update (#187)
- *(dev)* Set version to 0.1.19-dev
## [0.1.18] - 2024-07-12

### ⚙️ Miscellaneous Tasks

- Admin panel base path only on local (#186)
- *(dev)* Set version to 0.1.18-dev
## [0.1.17] - 2024-07-12

### 🚀 Features

- Health check admin panel (#185)
- Create loans from admin-panel (#183)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.17-dev
## [0.1.16] - 2024-07-11

### 🚀 Features

- Log 500x errors on server (#182)
- Show user loans (#179)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.16-dev
## [0.1.15] - 2024-07-11

### 🚀 Features

- Ui for chart of accounts (#165)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.15-dev
## [0.1.14] - 2024-07-10

### 🚀 Features

- Add auth for admin panel and admin API (#159)
- Add chart of accounts queries (#154)

### 🐛 Bug Fixes

- When lower/uppercase mistmatch (#160)
- Frontend flow (#174)
- Order by l.id (#178)

### 🚜 Refactor

- Use react-icons-lib (#162)
- Simplify chart-of-accounts schema types (#177)

### 🧪 Testing

- Customer portal e2e auth tests (#169)

### ⚙️ Miscellaneous Tasks

- Adding next-runtime-end (#155)
- *(dev)* Set version to 0.1.14-dev
- Expose termValuesCreate mutation (#157)
- Adding sumsub on frontend (#153)
- Loan create mutation (#158)
- Display added passkeys (#156)
- Change trial balance net to net_debit (#135)
- Adding gitkeep for public (#168)
- Remove fixed_term_loan (#164)
- Loan partial payment (#163)
- Remove fixed term loan (#171)
## [0.1.13] - 2024-07-05

### 🚀 Features

- Loan screen designs (#136)
- Sumsub integration (#149)

### ⚙️ Miscellaneous Tasks

- *(dev)* Set version to 0.1.13-dev
- Some ui improvements for trial-balance (#147)
- Adding auto submit for otp (#148)
- Adding sessions/whoami to rules (#150)
- Integrate 'me' query with frontend (#151)
- Loan instantiation (#144)
## [0.1.12] - 2024-07-02

### 🚀 Features

- 2fa with totp (#124)
- 2fa with webauth (#128)
- Add new "all layers" balance view (#139)
- Trial balance view in admin panel (#142)

### 🚜 Refactor

- Use api on client/browser (#138)

### ⚙️ Miscellaneous Tasks

- Remove apple builds because unused, untested and breaking ci (#134)
- *(dev)* Set version to 0.1.12-dev
- Adding health endpoint (#141)
- Loan structure (#140)
- Bump cala
## [0.1.11] - 2024-06-29

### 🚀 Features

- Using kratos to create user (#72)
- Move to use code instead of password for registration (#99)
- Initializing-frontend (#25)
- Adding primitive components (#103)
- Use JWT instead of header (#102)
- Add equity admin operation for external cash (#109)
- Adding authentication (#111)
- Add equity to balance sheet (#112)
- Dockerize customer portal (#115)
- Add trial balance query (#121)
- Portal and kratos through oathkeeper (#125)

### 🚜 Refactor

- Remove EntityUpdate from codebase
- Some auth improvements (#96)
- Create bfx integrations from tf (#108)
- Trial balance types and naming (#130)

### 🧪 Testing

- Unauthenticated -> unauthorized
- Assert_assets_liabilities in user.bats and fixed_term_loan.bats (#110)

### ⚙️ Miscellaneous Tasks

- *(admin-panel)* Rename user details in table view (#94)
- *(dev)* Set version to 0.1.11-dev
- Adding github check-code for apps (#104)
- Refactor release channel for admin-panel (#105)
- Make lava gql queries more idiomatic (#98)
- Misc renaming bats-test to dockerhost-alias (#100)
- Bootstrap USDT_CACHE via tf (#97)
- Bump cala schema
- Rename lava bank front-end (#113)
- *(shared)* Bump vendored ci files (#117)
- *(shared)* Bump vendored ci files (#118)
- *(shared)* Bump vendored ci files (#119)
- Bump cala
- Renaming settings to user - withdraw - loan (#120)
- Bump cala schema and add latest changes (#127)
- Bump cala (#132)
- Bump cala (#133)
## [0.1.10] - 2024-06-13

### ⚙️ Miscellaneous Tasks

- *(admin-panel)* Rename title for user details (#93)
- *(dev)* Set version to 0.1.10-dev
## [0.1.7] - 2024-06-13

### 🚀 Features

- Swap withdraw mutation implementation for bfx integration (#89)

### ⚙️ Miscellaneous Tasks

- *(admin-panel)* Ui improvements (#90)
- Old digest for bumps (#91)
## [0.1.6] - 2024-06-12

### ⚙️ Miscellaneous Tasks

- *(admin-panel)* Remove mutations/user actions (#86)
- Release dockerfile (#87)
## [0.1.5] - 2024-06-12

### 🚀 Features

- Add loansForUser query
- Add user deposit mutation (#50)
- Impl users query in admin server (#60)
- Admin panel loan screen (#73)
- Adding user screen (#75)
- *(admin-panel)* Added WithdrawalSettle action (#85)
- Bfx integration (#84)

### 🐛 Bug Fixes

- Write_public_sdl
- List_by_user query
- Map CouldNotFindById error to None (#57)

### 🚜 Refactor

- Split out mutations between admin and public

### 🧪 Testing

- Fix typo in GQL_ADMIN_ENDPOINT

### ⚙️ Miscellaneous Tasks

- Fixing merge conflicts
- Update legacy code
- Output schema to public module
- Move settle_withdrawal.gql from gql to admin-gql
- Include existing queries in admin server
- LoansForUser can be null
- Include cala-enterprise (#58)
- Configure-docker-in-ci
- Initializing admin panel (#62)
- Update fetch cala schema (#65)
- Bump cala-types (#68)
- Move common graphql elements to shared dir (#56)
- Add apollo studio (#51)
- Adding primitive components (#64)
- Add admin-panel dockerfile and ci to build it (#74)
- Bump cala schema (#77)
- Bump cala schema (#78)
- *(admin-panel)* Formatting currency (#76)
- Dockerfile for admin-panel (#79)
- Fix pulls (#80)
- Admin panel release image (#82)
- *(admin-panel)* Remove apollo wrapper (#83)
## [0.1.4] - 2024-06-04

### 🚀 Features

- Add completed event to loan entity
- Add complete-loan logic at ledger layer
- Add complete-loan mutation at gql layer

### 🚜 Refactor

- Add amount field to Withdrawal entity
- Rename Core Assets account
- Change loan event to FullyRepaid
- Move explicit loan-complete to repayment operation
- Combine final-payment and release-collateral steps
- CompleteLoan template

### 🧪 Testing

- Add complete-loan bats tests

### ⚙️ Miscellaneous Tasks

- Change withdrawal payload id field name
- Add repaid check for loan completion
## [0.1.3] - 2024-06-03

### 🚀 Features

- Add withdraw mutations
- Add withdraw template create
- Add 'user' query
- Add settle-withdrawal ledger function
- Add 'WithdrawalInitiated' user event
- Add 'settle_withdrawal' use case
- Add withdraw entity
- Add 'settle_withdrawal' graphql mutation

### 🐛 Bug Fixes

- Add externalId to withdraw params

### 🚜 Refactor

- Remove ach mutation
- Change 'withdraw' to 'initiate withdrawal'
- Reorder ledger mod fns
- Add encumbrance layer to user checking balance
- Move LayeredBalance to ledger primitives
- Remove CurrencyAmount enum
- Move withdrawal methods from user to withdraw entity

### 🧪 Testing

- Add user withdraw bats tests
- Refactor withdraw test
- Wait for now server connection errors on start
- Change numbers

### ⚙️ Miscellaneous Tasks

- Update charts image (#37)
- Fix path of chart values update (#38)
- Add sqlx-prepare make command
- Add Withdraws to LavaApp
- Rename withdraw mutations to reflect entity relationship
- Simplify withdraw model
- Withdraw via pending
- Add integration tests (#39)
- Refactoring to public folder
- Extract out server primitives in shared module
- Remove LAVA_SERVER_ID env variable
- Remove redundant makefile commands
- Remove executing_server_id from job_executions table
- Make poll_jobs a trace
- Bump image in concourse
- Remove redundant example fn
- Rename topup to pledge
- Rename to withdrawal at gql layer
- Bump cala
## [0.1.2] - 2024-06-01

### 🐛 Bug Fixes

- Lava name
- Dockerfile for lava-bank (#36)
## [0.1.1] - 2024-05-29

### 🚀 Features

- Add DepositAccount to graphql
- Add create-standard-templates ledger method
- Initialise standard templates in ledger
- Add version to transaction template structs
- Add ledger 'find_tx_template_by_code' method
- Approve loan e2e
- Add loan principal usd entries to template
- Implement incur-interest tx template
- Add balance field on FixedTermLoan gql entity
- Add 'make payment' mutation
- Check if loan is repaid on make-payment
- Handle making full repayment of loan

### 🐛 Bug Fixes

- Clippy
- Cargo sqlx prepare
- Clippy
- Assign correct normal_balance_type for created accounts
- Clippy

### 💼 Other

- Fix user.bats
- Add bump-cala docker image update script

### 🚜 Refactor

- Separate standard tx template mutations
- Change grouping of transactions gql operations
- Change grouping of transactions in cala module
- Assert tx template creation in ledger
- DepositAccount -> UnallocatedCollateral
- Ledger-account -> unallocated-collateral
- Deposit -> topup naming
- Gql files by concept
- Core-journal-create under journals.gql
- Return all balances for user in gql
- Get_user_balance
- Add normal_balance_type param to account creation
- Rename tx -> db
- Combine loan principal & interest into 'outstanding' account
- Rename interest function
- Debit user checking account for loan payments instead
- Remove 'excess payment amount' handling
- Make_payment -> record_payment
- Tell don't ask

### 🧪 Testing

- Fix user.bats
- E2e topup  unallocated collateral
- Update fixed-term-loan.bats
- Check loan balances in lifecycle
- Check for loan transaction after loan approve
- Change mutation name after refactor
- Bump num retries

### ⚙️ Miscellaneous Tasks

- Line of credit boilerplate
- Add 'LAVA_ASSETS_ACCOUNT_ID' constant
- Fix lints
- Renaming deposit to topup user unallocated collateral
- Adding external mutation
- Tweak topup template
- Implement execute topup
- Execute topup template
- Assert CORE_ASSETS account exsts
- Remove fixedTermLoanDeclareCollateralized
- Initialize loans with account ids
- Delete LineOfCredit stuff
- Persist user_id in loan
- Remove lava-accounts-create.gql
- Rename LAVA_JOURNAL_ID -> CORE_JOURNAL_ID
- Pass reference for transaction
- Remove redundant files
- Remove unused UnallocatedCollateral structs
- Interest job wip
- Interest job boilerplate
- Spawn in approve
- Bump cala schema
- Remove executable mode
- Bump cala
- Switch ports for cala / lava for easier running of local cala
## [0.1.0] - 2024-05-21

### 🐛 Bug Fixes

- Sqlx prepare
- Typo
- Clippy

### 🧪 Testing

- Initial bats test
- Fix config location
- Export LAVA_CONFIG

### ⚙️ Miscellaneous Tasks

- Initial commit
- Core boilerplate
- Server / gql / app boilerplate
- Dummy schema
- Add entity framework
- Fixed_term_loan_boilerplate
- More fixed_term_loan boilerplate
- Create fixed term loan
- Gql for create loan
- E2e loan
- Dummy Ledger scaffolding
- Import job
- More job boilerplate
- Fixed_term_loan::Job
- Introduce FixedTermLoanState
- Add collateralization
- Add Collateralized
- Shout out to anonym super rockstar coder (who is it?)
- Purge some deps
- Add .cargo
- Get tracing working
- Otel stuff
- Cala integration boilerplate
- Boilerplate to create account
- Actually create loan account
- Update cala schema
- Bump some boilerplate
- Exctract job_execution
- Implement Pause
- Get balance boilerplate
- Create journal in startup
- Expose balance on fixedTermLoan
- Remove openssl dep for graphql_client
- User boilerplate
- Add user-entity
- *(shared)* Bump vendored ci files (#11)
- Bump schema
- Introduce release pipeline (#12)
